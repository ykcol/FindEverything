use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use ignore::{WalkBuilder, DirEntry};
use indicatif::{ProgressBar, ProgressStyle};

// 使用infrastructure层的LoggerTrait
use crate::infrastructure::LoggerTrait;

/// 文件筛选条件
#[derive(Debug, Clone)]
pub struct FileFilter {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub excluded_dirs: HashSet<String>,
    pub excluded_paths: HashSet<String>,
}

impl FileFilter {
    /// 创建新的文件过滤器
    pub fn new(
        min_size: Option<u64>,
        max_size: Option<u64>,
        excluded_dirs: Vec<String>,
        excluded_paths: Vec<String>,
    ) -> Self {
        Self {
            min_size,
            max_size,
            excluded_dirs: excluded_dirs.into_iter().collect(),
            excluded_paths: excluded_paths.into_iter().collect(),
        }
    }

    /// 检查文件是否符合大小要求
    pub fn matches_size(&self, size: u64) -> bool {
        let min_ok = self.min_size.map_or(true, |min| size >= min);
        let max_ok = self.max_size.map_or(true, |max| size <= max);
        min_ok && max_ok
    }

    /// 检查路径是否被排除
    pub fn is_path_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let normalized_path = path_str.replace('\\', "/");
        
        // 检查是否在排除路径列表中（支持多种匹配方式）
        for excluded_path in &self.excluded_paths {
            let normalized_excluded = excluded_path.replace('\\', "/");
            
            // 精确匹配
            if normalized_path == normalized_excluded {
                return true;
            }
            
            // 文件名匹配
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy() == *excluded_path {
                    return true;
                }
            }
            
            // 路径结尾匹配
            if normalized_path.ends_with(&normalized_excluded) {
                return true;
            }
        }
        
        // 检查是否在排除目录中
        for component in path.components() {
            if let Some(dir_name) = component.as_os_str().to_str() {
                if self.excluded_dirs.contains(dir_name) {
                    return true;
                }
            }
        }
        
        false
    }

    /// 检查文件是否应该被处理
    pub fn should_process(&self, entry: &DirEntry) -> Result<bool> {
        // 检查路径排除
        if self.is_path_excluded(entry.path()) {
            return Ok(false);
        }

        // 检查文件大小
        if let Ok(metadata) = entry.metadata() {
            if !self.matches_size(metadata.len()) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// 向后兼容的类型别名
pub type FileSizeFilter = FileFilter;

/// 扫描并执行回调函数处理文件
pub fn scan_directory<F>(
    dir: &Path,
    filter: FileSizeFilter,
    parallel: bool,
    respect_gitignore: bool,
    logger: Arc<dyn LoggerTrait>,
    callback: F,
) -> Result<(u64, u64)>
where
    F: Fn(&DirEntry) -> Result<()> + Send + Sync + 'static,
{
    let callback = Arc::new(callback);
    let filter = Arc::new(filter);
    let total_files = Arc::new(AtomicU64::new(0));
    let processed_files = Arc::new(AtomicU64::new(0));

    // 创建进度条
    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
    );
    progress.set_message("已处理 0 文件");

    // 创建文件遍历器
    let mut walker = WalkBuilder::new(dir);
    walker
        .hidden(false)
        .git_ignore(respect_gitignore)
        .git_global(respect_gitignore)
        .git_exclude(respect_gitignore);

    if parallel {
        walker.threads(num_cpus::get());
    } else {
        walker.threads(1);
    }

    // 执行并行遍历
    let logger_clone = Arc::clone(&logger);
    walker.build_parallel().run(|| {
        let callback = Arc::clone(&callback);
        let filter = Arc::clone(&filter);
        let total_files = Arc::clone(&total_files);
        let processed_files = Arc::clone(&processed_files);
        let logger = Arc::clone(&logger_clone);
        let progress = progress.clone();

        Box::new(move |result| {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    // 记录遍历错误
                    if logger.is_enabled() {
                        let _ = logger.log_message(&format!("遍历错误: {}", err));
                    }
                    return ignore::WalkState::Continue;
                }
            };

            // 只处理文件
            if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                return ignore::WalkState::Continue;
            }
            
            // 检查是否应该处理此文件（包括排除规则和大小过滤）
            match filter.should_process(&entry) {
                Ok(false) => {
                    // 记录被过滤的文件
                    if logger.is_enabled() {
                        let reason = if filter.is_path_excluded(entry.path()) {
                            "已跳过(路径排除)"
                        } else {
                            "已跳过(大小过滤)"
                        };
                        
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let _ = logger.log_file(entry.path(), size, reason);
                    }
                    return ignore::WalkState::Continue;
                }
                Err(err) => {
                    // 记录错误
                    if logger.is_enabled() {
                        let _ = logger.log_message(
                            &format!("检查文件过滤条件失败 {}: {}", entry.path().display(), err)
                        );
                    }
                    return ignore::WalkState::Continue;
                }
                Ok(true) => {
                    // 继续处理
                }
            }
            
            // 获取文件大小用于日志记录
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            // 更新计数器
            let current_total = total_files.fetch_add(1, Ordering::Relaxed) + 1;
            
            // 更新进度条
            progress.set_message(format!("已处理 {} 文件", current_total));
            progress.tick();

            // 记录文件处理
            if logger.is_enabled() {
                let _ = logger.log_file(entry.path(), size, "正在处理");
            }

            // 执行回调函数
            if let Err(err) = callback(&entry) {
                // 记录回调错误
                if logger.is_enabled() {
                    let _ = logger.log_message(
                        &format!("处理文件失败 {}: {}", entry.path().display(), err)
                    );
                }
            } else {
                processed_files.fetch_add(1, Ordering::Relaxed);
            }

            ignore::WalkState::Continue
        })
    });

    // 完成进度条
    let final_total = total_files.load(Ordering::Relaxed);
    let final_processed = processed_files.load(Ordering::Relaxed);
    
    progress.finish_with_message(format!("完成! 已处理 {} 文件", final_total));

    Ok((final_total, final_processed))
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_filter_creation() {
        let filter = FileFilter::new(
            Some(1024),
            Some(1048576),
            vec!["target".to_string()],
            vec!["test.txt".to_string()],
        );
        
        assert_eq!(filter.min_size, Some(1024));
        assert_eq!(filter.max_size, Some(1048576));
        assert!(filter.excluded_dirs.contains("target"));
        assert!(filter.excluded_paths.contains("test.txt"));
    }

    #[test]
    fn test_size_filtering() {
        let filter = FileFilter::new(Some(100), Some(1000), vec![], vec![]);
        
        assert!(!filter.matches_size(50));   // 太小
        assert!(filter.matches_size(500));   // 合适
        assert!(!filter.matches_size(2000)); // 太大
    }

    #[test]
    fn test_path_exclusion() {
        let filter = FileFilter::new(
            None,
            None,
            vec!["target".to_string()],
            vec!["test.txt".to_string()],
        );
        
        assert!(filter.is_path_excluded(&PathBuf::from("target/debug/app")));
        assert!(filter.is_path_excluded(&PathBuf::from("test.txt")));
        assert!(!filter.is_path_excluded(&PathBuf::from("src/main.rs")));
    }
}
