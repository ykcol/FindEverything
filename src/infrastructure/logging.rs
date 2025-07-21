use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use chrono::Local;

/// 日志记录器trait
pub trait LoggerTrait: Send + Sync {
    fn is_enabled(&self) -> bool;
    fn log_message(&self, message: &str) -> Result<()>;
    fn log_file(&self, path: &Path, size: u64, status: &str) -> Result<()>;
    fn finalize(&self, total_files: u64, matched_files: u64, total_matches: u64, duration: std::time::Duration) -> Result<()>;
}

/// 调试日志记录器（用于系统状态和调试信息）
pub struct Logger {
    log_file: Arc<Mutex<Option<File>>>,
    enabled: bool,
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new(enabled: bool) -> Result<Self> {
        if !enabled {
            return Ok(Self {
                log_file: Arc::new(Mutex::new(None)),
                enabled: false,
            });
        }

        // 获取当前时间作为文件名的一部分
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        
        // 构建调试日志文件路径 - 与程序同级目录
        let log_path = PathBuf::from(format!("debug_{}.log", timestamp));
        
        // 创建日志文件
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&log_path)?;
            
        // 写入UTF-8 BOM以确保文件被正确识别为UTF-8
        let mut file_clone = file.try_clone()?;
        file_clone.write_all(&[0xEF, 0xBB, 0xBF])?; // UTF-8 BOM
            
        // 写入调试日志头部信息
        writeln!(file_clone, "# FindEverything 调试日志")?;
        writeln!(file_clone, "# 开始时间: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file_clone, "# --------------------------------------------")?;
        writeln!(file_clone, "# 系统状态、配置信息和调试信息")?;
        
        Ok(Self {
            log_file: Arc::new(Mutex::new(Some(file))),
            enabled: true,
        })
    }


}

impl LoggerTrait for Logger {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn log_message(&self, message: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
        
        if let Ok(mut file_guard) = self.log_file.lock() {
            if let Some(ref mut file) = *file_guard {
                writeln!(file, "[{}] {}", timestamp, message)?;
                file.flush()?;
            }
        }
        
        Ok(())
    }

    fn log_file(&self, path: &Path, size: u64, status: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");
        
        if let Ok(mut file_guard) = self.log_file.lock() {
            if let Some(ref mut file) = *file_guard {
                writeln!(file, "[{}] 文件: {} | 大小: {} 字节 | 状态: {}", 
                    timestamp, 
                    path.display(), 
                    size, 
                    status
                )?;
                file.flush()?;
            }
        }
        
        Ok(())
    }

    fn finalize(&self, total_files: u64, matched_files: u64, total_matches: u64, duration: std::time::Duration) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let now = Local::now();
        
        if let Ok(mut file_guard) = self.log_file.lock() {
            if let Some(ref mut file) = *file_guard {
                writeln!(file, "# --------------------------------------------")?;
                writeln!(file, "# 搜索完成时间: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
                writeln!(file, "# 总用时: {:.3}秒", duration.as_secs_f64())?;
                writeln!(file, "# 扫描文件数: {}", total_files)?;
                writeln!(file, "# 匹配文件数: {}", matched_files)?;
                writeln!(file, "# 匹配项总数: {}", total_matches)?;
                writeln!(file, "# ============================================")?;
                file.flush()?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(false).unwrap();
        assert!(!logger.is_enabled());
        
        let logger = Logger::new(true).unwrap();
        assert!(logger.is_enabled());
    }

    #[test]
    fn test_logger_trait() {
        let logger = Logger::new(true).unwrap();
        let logger_trait: &dyn LoggerTrait = &logger;
        
        assert!(logger_trait.is_enabled());
        assert!(logger_trait.log_message("test message").is_ok());
    }
}
