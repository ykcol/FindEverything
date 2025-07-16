use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use ignore::{WalkBuilder, DirEntry};
use indicatif::{ProgressBar, ProgressStyle};

use crate::logger::Logger;

/// 文件筛选条件
#[derive(Debug, Clone, Copy)]
pub struct FileSizeFilter {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

impl FileSizeFilter {
    /// 检查文件是否符合大小要求
    pub fn matches(&self, size: u64) -> bool {
        let min_ok = self.min_size.map_or(true, |min| size >= min);
        let max_ok = self.max_size.map_or(true, |max| size <= max);
        min_ok && max_ok
    }
}

/// 扫描并执行回调函数处理文件
pub fn scan_directory<F>(
    dir: &Path,
    filter: FileSizeFilter,
    parallel: bool,
    respect_gitignore: bool,
    logger: Arc<Logger>,
    callback: F,
) -> Result<(u64, u64)>
where
    F: Fn(&DirEntry) -> Result<()> + Send + Sync + 'static,
{
    let callback = Arc::new(callback);
    let total_files = Arc::new(AtomicU64::new(0));
    let processed_files = Arc::new(AtomicU64::new(0));
    
    // 创建进度条
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .expect("无效的进度条模板")
    );
    
    // 记录日志
    if logger.is_enabled() {
        logger.log_message(&format!("开始扫描目录: {}", dir.display()))?;
    }
    
    // 创建遍历器
    let mut builder = WalkBuilder::new(dir);
    builder
        .hidden(false) // 包含隐藏文件
        .follow_links(false) // 不跟随符号链接
        .git_global(respect_gitignore) // 是否使用全局 .gitignore
        .git_ignore(respect_gitignore) // 是否使用 .gitignore
        .git_exclude(respect_gitignore) // 是否使用 .git/info/exclude
        .threads(if parallel {
            num_cpus::get()
        } else {
            1
        });
    
    // 开始遍历
    let walker = builder.build_parallel();
    
    // 启动进度更新线程
    let progress_clone = progress.clone();
    let processed_files_clone = Arc::clone(&processed_files);
    let total_files_clone = Arc::clone(&total_files);
    let logger_clone = Arc::clone(&logger);
    
    std::thread::spawn(move || {
        loop {
            let processed = processed_files_clone.load(Ordering::Relaxed);
            let total = total_files_clone.load(Ordering::Relaxed);
            
            if total > 0 {
                let percentage = (processed as f64 / total as f64) * 100.0;
                let message = format!(
                    "已处理 {}/{} 文件 ({:.1}%)",
                    processed, total, percentage
                );
                progress_clone.set_message(message.clone());
                
                // 记录进度到日志
                if logger_clone.is_enabled() && processed % 100 == 0 {
                    let _ = logger_clone.log_message(&message);
                }
            } else {
                progress_clone.set_message(format!("已处理 {} 文件", processed));
            }
            
            std::thread::sleep(Duration::from_millis(100));
            
            if processed > 0 && processed == total {
                break;
            }
        }
        
        let final_message = format!(
            "完成! 已处理 {} 文件",
            processed_files_clone.load(Ordering::Relaxed)
        );
        progress_clone.finish_with_message(final_message.clone());
        
        // 记录完成信息
        if logger_clone.is_enabled() {
            let _ = logger_clone.log_message(&final_message);
        }
    });
    
    // 执行并行遍历
    let logger_clone = Arc::clone(&logger);
    walker.run(|| {
        let callback = Arc::clone(&callback);
        let total_files = Arc::clone(&total_files);
        let processed_files = Arc::clone(&processed_files);
        let logger = Arc::clone(&logger_clone);
        
        Box::new(move |entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    // 记录错误
                    if logger.is_enabled() {
                        let _ = logger.log_message(&format!("错误: {}", err));
                    }
                    return ignore::WalkState::Continue;
                },
            };
            
            // 只处理文件
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                return ignore::WalkState::Continue;
            }
            
            // 获取文件大小
            let metadata = match std::fs::metadata(entry.path()) {
                Ok(metadata) => metadata,
                Err(err) => {
                    // 记录错误
                    if logger.is_enabled() {
                        let _ = logger.log_message(
                            &format!("无法获取文件元数据 {}: {}", entry.path().display(), err)
                        );
                    }
                    return ignore::WalkState::Continue;
                },
            };
            
            let size = metadata.len();
            
            // 应用文件大小过滤
            if !filter.matches(size) {
                // 记录被过滤的文件
                if logger.is_enabled() {
                    let _ = logger.log_file(
                        entry.path(),
                        size,
                        &format!("已跳过(大小过滤: {}字节)", size)
                    );
                }
                return ignore::WalkState::Continue;
            }
            
            // 记录扫描文件
            if logger.is_enabled() {
                let _ = logger.log_file(entry.path(), size, "扫描中");
            }
            
            // 计数总文件数
            total_files.fetch_add(1, Ordering::Relaxed);
            
            // 执行回调
            if let Err(err) = callback(&entry) {
                //eprintln!("处理文件失败 {}: {}", entry.path().display(), err);
                // 记录错误
                if logger.is_enabled() {
                    let _ = logger.log_file(entry.path(), size, &format!("错误: {}", err));
                }
            } else if logger.is_enabled() {
                // 记录成功处理
                let _ = logger.log_file(entry.path(), size, "已处理");
            }
            
            // 更新已处理文件计数
            processed_files.fetch_add(1, Ordering::Relaxed);
            
            ignore::WalkState::Continue
        })
    });
    
    // 等待进度条线程完成
    std::thread::sleep(Duration::from_millis(200));
    progress.finish();
    
    Ok((
        total_files.load(Ordering::Relaxed),
        processed_files.load(Ordering::Relaxed),
    ))
} 