use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use sysinfo::System;

use crate::application::Config;
use crate::infrastructure::{LoggerTrait, Logger};

/// 监控trait
pub trait MonitoringTrait: Send + Sync {
    fn start(&self) -> Result<()>;
    fn stop(&self);
    fn apply_throttle(&self);
    fn get_cpu_usage(&self) -> f32;
    fn should_throttle(&self) -> bool;
    fn get_status(&self) -> MonitorStatus;
}

/// CPU监控器
pub struct CpuMonitor {
    cpu_threshold: f32,
    search_delay_ms: u64,
    current_cpu_usage: Arc<AtomicU64>, // 存储CPU使用率 * 100
    should_throttle: Arc<AtomicBool>,
    is_running: Arc<AtomicBool>,
    logger: Arc<Logger>,
}

impl CpuMonitor {
    /// 创建新的CPU监控器
    pub fn new(config: &Config, logger: Arc<Logger>) -> Self {
        Self {
            cpu_threshold: config.performance.cpu_threshold,
            search_delay_ms: config.performance.search_delay_ms,
            current_cpu_usage: Arc::new(AtomicU64::new(0)),
            should_throttle: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
            logger,
        }
    }
}

impl MonitoringTrait for CpuMonitor {
    fn start(&self) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.is_running.store(true, Ordering::Relaxed);
        
        let cpu_threshold = self.cpu_threshold;
        let current_cpu_usage = Arc::clone(&self.current_cpu_usage);
        let should_throttle = Arc::clone(&self.should_throttle);
        let is_running = Arc::clone(&self.is_running);
        let logger = Arc::clone(&self.logger);

        thread::spawn(move || {
            let mut system = System::new_all();
            let mut last_log_time = Instant::now();
            
            while is_running.load(Ordering::Relaxed) {
                system.refresh_cpu();
                
                // 计算平均CPU使用率
                let cpu_usage = system.cpus().iter()
                    .map(|cpu| cpu.cpu_usage())
                    .sum::<f32>() / system.cpus().len() as f32;
                
                // 存储CPU使用率（乘以100以便用整数存储）
                current_cpu_usage.store((cpu_usage * 100.0) as u64, Ordering::Relaxed);
                
                // 检查是否需要限流
                let needs_throttle = cpu_usage > cpu_threshold;
                should_throttle.store(needs_throttle, Ordering::Relaxed);
                
                // 每5秒记录一次CPU使用率
                if logger.is_enabled() && last_log_time.elapsed() >= Duration::from_secs(5) {
                    let status = if needs_throttle { "限流中" } else { "正常" };
                    let _ = logger.log_message(&format!(
                        "CPU使用率: {:.1}% (阈值: {:.1}%) - {}",
                        cpu_usage, cpu_threshold, status
                    ));
                    last_log_time = Instant::now();
                }
                
                thread::sleep(Duration::from_secs(1));
            }
        });

        if self.logger.is_enabled() {
            self.logger.log_message(&format!(
                "CPU监控已启动 - 阈值: {:.1}%, 延迟: {}ms",
                self.cpu_threshold, self.search_delay_ms
            ))?;
        }

        Ok(())
    }

    fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        
        if self.logger.is_enabled() {
            let _ = self.logger.log_message("CPU监控已停止");
        }
    }

    fn get_cpu_usage(&self) -> f32 {
        self.current_cpu_usage.load(Ordering::Relaxed) as f32 / 100.0
    }

    fn should_throttle(&self) -> bool {
        self.should_throttle.load(Ordering::Relaxed)
    }

    fn apply_throttle(&self) {
        if self.should_throttle() {
            thread::sleep(Duration::from_millis(self.search_delay_ms));
        }
    }

    fn get_status(&self) -> MonitorStatus {
        MonitorStatus {
            cpu_usage: self.get_cpu_usage(),
            cpu_threshold: self.cpu_threshold,
            is_throttling: self.should_throttle(),
            is_running: self.is_running.load(Ordering::Relaxed),
        }
    }
}

impl Drop for CpuMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 监控状态信息
#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub cpu_usage: f32,
    pub cpu_threshold: f32,
    pub is_throttling: bool,
    pub is_running: bool,
}

impl MonitorStatus {
    /// 格式化状态信息
    pub fn format(&self) -> String {
        format!(
            "CPU: {:.1}%/{:.1}% {}{}",
            self.cpu_usage,
            self.cpu_threshold,
            if self.is_throttling { "(限流)" } else { "(正常)" },
            if self.is_running { "" } else { " [已停止]" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::Config;
    use crate::infrastructure::Logger;

    #[test]
    fn test_cpu_monitor_creation() {
        let config = Config::default();
        let logger = Arc::new(Logger::new(false).unwrap());
        let monitor = CpuMonitor::new(&config, logger);
        
        assert_eq!(monitor.cpu_threshold, 80.0);
        assert_eq!(monitor.search_delay_ms, 100);
        assert!(!monitor.should_throttle());
    }

    #[test]
    fn test_monitor_status() {
        let config = Config::default();
        let logger = Arc::new(Logger::new(false).unwrap());
        let monitor = CpuMonitor::new(&config, logger);
        
        let status = monitor.get_status();
        assert_eq!(status.cpu_threshold, 80.0);
        assert!(!status.is_throttling);
        
        let formatted = status.format();
        assert!(formatted.contains("CPU:"));
        assert!(formatted.contains("(正常)"));
    }
}
