use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use anyhow::Result;
use chrono::Local;

/// 错误类型分类
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// 文件读取错误
    FileRead,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::FileRead => "文件读取",
        }
    }
}



/// 错误日志记录器
pub struct ErrorLogger {
    error_file: Arc<Mutex<Option<File>>>,
    error_path: PathBuf,
    enabled: bool,
    error_counts: Arc<Mutex<HashMap<ErrorType, usize>>>,
}

impl ErrorLogger {
    /// 创建新的错误日志记录器
    pub fn new(enabled: bool) -> Result<Self> {
        if !enabled {
            return Ok(Self {
                error_file: Arc::new(Mutex::new(None)),
                error_path: PathBuf::new(),
                enabled: false,
                error_counts: Arc::new(Mutex::new(HashMap::new())),
            });
        }

        // 获取当前时间作为文件名的一部分
        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        
        // 构建错误日志文件路径 - 与程序同级目录
        let error_path = PathBuf::from(format!("error_{}.log", timestamp));
        
        // 创建错误日志文件
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&error_path)?;
            
        // 写入UTF-8 BOM以确保文件被正确识别为UTF-8
        let mut file_clone = file.try_clone()?;
        file_clone.write_all(&[0xEF, 0xBB, 0xBF])?; // UTF-8 BOM
            
        // 写入错误日志头部信息
        writeln!(file_clone, "# FindEverything 错误日志")?;
        writeln!(file_clone, "# 开始时间: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file_clone, "# ============================================")?;
        writeln!(file_clone)?;
        
        Ok(Self {
            error_file: Arc::new(Mutex::new(Some(file))),
            error_path,
            enabled: true,
            error_counts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// 记录错误
    pub fn log_error(
        &self,
        error_type: ErrorType,
        file_path: Option<&str>,
        message: &str,
        details: Option<&str>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");

        // 更新错误计数
        {
            let mut counts = self.error_counts.lock().unwrap();
            *counts.entry(error_type.clone()).or_insert(0) += 1;
        }

        // 写入错误日志文件
        if let Ok(mut file_guard) = self.error_file.lock() {
            if let Some(ref mut file) = *file_guard {
                writeln!(file, "[{}] {} - {}", 
                    timestamp, 
                    error_type.as_str(), 
                    message
                )?;
                
                if let Some(path) = file_path {
                    writeln!(file, "  文件路径: {}", path)?;
                }
                
                if let Some(detail) = details {
                    writeln!(file, "  详细信息: {}", detail)?;
                }
                
                writeln!(file)?; // 空行分隔
                file.flush()?;
            }
        }

        Ok(())
    }

    /// 获取错误统计信息
    pub fn get_error_summary(&self) -> HashMap<ErrorType, usize> {
        if let Ok(counts) = self.error_counts.lock() {
            counts.clone()
        } else {
            HashMap::new()
        }
    }

    /// 获取总错误数
    pub fn get_total_errors(&self) -> usize {
        if let Ok(counts) = self.error_counts.lock() {
            counts.values().sum()
        } else {
            0
        }
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        self.get_total_errors() > 0
    }



    /// 完成错误日志记录
    pub fn finalize(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        if let Ok(mut file_guard) = self.error_file.lock() {
            if let Some(ref mut file) = *file_guard {
                let now = Local::now();
                writeln!(file, "# ============================================")?;
                writeln!(file, "# 结束时间: {}", now.format("%Y-%m-%d %H:%M:%S"))?;
                
                let summary = self.get_error_summary();
                if !summary.is_empty() {
                    writeln!(file, "# 错误统计:")?;
                    for (error_type, count) in &summary {
                        writeln!(file, "#   {}: {} 次", error_type.as_str(), count)?;
                    }
                    writeln!(file, "#   总计: {} 个错误", self.get_total_errors())?;
                } else {
                    writeln!(file, "# 无错误记录")?;
                }
                
                file.flush()?;
            }
        }

        Ok(())
    }

    /// 打印错误摘要到控制台
    pub fn print_error_summary(&self) {
        if !self.has_errors() {
            return;
        }

        println!("\n⚠️  搜索过程中发现错误:");
        println!("----------------------------");
        
        let summary = self.get_error_summary();
        for (error_type, count) in &summary {
            println!("  {}: {} 次", error_type.as_str(), count);
        }
        
        println!("  总计: {} 个错误", self.get_total_errors());
        println!("  详细错误信息请查看: {}", self.error_path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_error_logger_creation() {
        let logger = ErrorLogger::new(false).unwrap();
        assert_eq!(logger.get_total_errors(), 0);
    }

    #[test]
    fn test_error_logging() {
        let logger = ErrorLogger::new(true).unwrap();

        logger.log_error(
            ErrorType::FileRead,
            Some("/test/path"),
            "测试错误",
            Some("详细信息")
        ).unwrap();

        assert_eq!(logger.get_total_errors(), 1);
        assert!(logger.has_errors());

        let summary = logger.get_error_summary();
        assert_eq!(summary.get(&ErrorType::FileRead), Some(&1));
    }

    #[test]
    fn test_error_types() {
        assert_eq!(ErrorType::FileRead.as_str(), "文件读取");
    }
}
