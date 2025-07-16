use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use chrono::Local;

/// 日志记录器
pub struct Logger {
    log_file: Arc<Mutex<Option<File>>>,
    log_path: PathBuf,
    enabled: bool,
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new(enabled: bool) -> Result<Self> {
        if !enabled {
            return Ok(Self {
                log_file: Arc::new(Mutex::new(None)),
                log_path: PathBuf::new(),
                enabled: false,
            });
        }

        // 获取当前时间作为文件名的一部分
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        
        // 构建日志文件路径 - 与程序同级目录
        let log_path = PathBuf::from(format!("findeverything_log_{}.txt", timestamp));
        
        // 创建日志文件
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)?;
            
        // 写入UTF-8 BOM以确保文件被正确识别为UTF-8
        let mut file_clone = file.try_clone()?;
        file_clone.write_all(&[0xEF, 0xBB, 0xBF])?; // UTF-8 BOM
            
        // 写入日志头部信息
        writeln!(file_clone, "# FindEverything 扫描日志")?;
        writeln!(file_clone, "# 开始时间: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file_clone, "# --------------------------------------------")?;
        writeln!(file_clone, "# 路径, 大小(字节), 状态")?;
        
        println!("日志文件已创建: {}", log_path.display());
        
        Ok(Self {
            log_file: Arc::new(Mutex::new(Some(file))),
            log_path,
            enabled,
        })
    }
    
    /// 记录扫描的文件
    pub fn log_file(&self, path: &Path, size: u64, status: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if let Some(file) = &mut *self.log_file.lock().unwrap() {
            writeln!(
                file, 
                "{}, {}, {}", 
                path.to_string_lossy(), 
                size, 
                status
            )?;
            // 确保立即写入磁盘
            file.flush()?;
        }
        
        Ok(())
    }
    
    /// 记录一般消息
    pub fn log_message(&self, message: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if let Some(file) = &mut *self.log_file.lock().unwrap() {
            writeln!(file, "# {}", message)?;
            file.flush()?;
        }
        
        Ok(())
    }
    
    /// 完成日志记录
    pub fn finalize(&self, total_files: u64, matched_files: u64, matched_items: u64, duration: std::time::Duration) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if let Some(file) = &mut *self.log_file.lock().unwrap() {
            let now = Local::now();
            
            writeln!(file)?;
            writeln!(file, "# --------------------------------------------")?;
            writeln!(file, "# 扫描完成")?;
            writeln!(file, "# 结束时间: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
            writeln!(file, "# 总扫描文件数: {}", total_files)?;
            writeln!(file, "# 匹配文件数: {}", matched_files)?;
            writeln!(file, "# 匹配项数: {}", matched_items)?;
            writeln!(file, "# 总耗时: {:?}", duration)?;
            
            file.flush()?;
        }
        
        if self.enabled {
            println!("完整日志已保存到: {}", self.log_path.display());
        }
        
        Ok(())
    }
    
    /// 获取日志文件路径
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }
    
    /// 检查日志是否已启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
} 