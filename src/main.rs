use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Parser;
use crossbeam_channel::bounded;

// 使用新的模块结构
mod domain;
mod application;
mod infrastructure;
mod presentation;

use application::Config;
use infrastructure::{Logger, ErrorLogger, ErrorType, CpuMonitor, LoggerTrait, MonitoringTrait};
use presentation::{SearchSummary, print_search_result};
use domain::{SearchPattern, SearchResult, FileFilter};

/// 查找文件内容的命令行工具
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 要搜索的内容
    #[clap(required = true)]
    pattern: String,
    
    /// 要搜索的目录路径
    #[clap()]
    path: Option<PathBuf>,
    
    /// 使用正则表达式搜索
    #[clap(short, long)]
    regex: bool,
    
    /// 将搜索内容解析为十六进制值
    #[clap(short = 'x', long)]
    hex: bool,
    
    /// 最小文件大小 (例如 "1K", "1M", "1G")
    #[clap(long)]
    min_size: Option<String>,
    
    /// 最大文件大小 (例如 "1K", "1M", "1G")
    #[clap(long)]
    max_size: Option<String>,
    
    /// 不使用并行处理 (默认使用所有可用CPU)
    #[clap(long)]
    no_parallel: bool,
    
    /// 启用详细日志记录，日志文件将保存到程序同级目录下
    #[clap(long)]
    log: bool,

    /// 排除指定目录（用逗号分隔）
    #[clap(long)]
    exclude_dir: Option<String>,

    /// 排除文件路径列表文件
    #[clap(long)]
    exclude_file: Option<PathBuf>,
}

/// 解析文件大小字符串为字节数
fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_lowercase();
    
    let multiplier = if size_str.ends_with('k') {
        1024
    } else if size_str.ends_with('m') {
        1024 * 1024
    } else if size_str.ends_with('g') {
        1024 * 1024 * 1024
    } else {
        1
    };
    
    let numeric_part = size_str
        .trim_end_matches(|c: char| c.is_alphabetic())
        .parse::<f64>()
        .context("无效的大小值")?;
    
    Ok((numeric_part * multiplier as f64) as u64)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 加载配置文件
    let config_path = Config::default_config_path()?;
    let config = Config::load_or_create(&config_path)?;
    config.validate()?;

    // 确定搜索路径（命令行参数优先于配置文件）
    let search_path = args.path.unwrap_or_else(|| {
        PathBuf::from(&config.search.default_search_path)
    });

    // 初始化日志记录器
    let logger = Arc::new(Logger::new(args.log)?);

    // 初始化错误日志记录器
    let error_logger = Arc::new(ErrorLogger::new(true)?); // 总是启用错误日志

    // 初始化CPU监控器
    let cpu_monitor = Arc::new(CpuMonitor::new(&config, Arc::clone(&logger)));
    cpu_monitor.start()?;

    // 解析搜索模式
    let pattern = SearchPattern::from_input(&args.pattern, args.regex, args.hex)?;
    let matcher = pattern.get_matcher()?;
    
    // 解析排除目录
    let mut excluded_dirs = config.exclude.default_dirs.clone();
    if let Some(exclude_dirs) = &args.exclude_dir {
        excluded_dirs.extend(
            exclude_dirs.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        );
    }

    // 解析排除文件路径
    let mut excluded_paths = config.exclude.default_files.clone();
    if let Some(exclude_file_path) = &args.exclude_file {
        if exclude_file_path.exists() {
            let content = std::fs::read_to_string(exclude_file_path)
                .with_context(|| format!("无法读取排除文件: {}", exclude_file_path.display()))?;

            excluded_paths.extend(
                content.lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
            );
        }
    }

    // 创建文件过滤器
    let filter = FileFilter::new(
        args.min_size.as_deref().map(parse_size).transpose()?,
        args.max_size.as_deref().map(parse_size).transpose()?,
        excluded_dirs,
        excluded_paths,
    );
    
    // 创建搜索摘要
    let summary = Arc::new(Mutex::new(SearchSummary::new()));
    
    // 存储已匹配文件路径
    let matched_files = Arc::new(Mutex::new(HashSet::new()));
    
    // 创建结果通道
    let (tx, rx) = bounded::<SearchResult>(100);
    
    // 创建处理线程
    let summary_clone = Arc::clone(&summary);
    let matched_files_clone = Arc::clone(&matched_files);
    let logger_clone = Arc::clone(&logger);
    
    let handle = std::thread::spawn(move || -> Result<()> {
        // 从通道接收并处理结果
        while let Ok(result) = rx.recv() {
            // 打印结果
            print_search_result(&result)?;
            
            // 更新统计信息
            let mut summary = summary_clone.lock().unwrap();
            let mut matched_paths = matched_files_clone.lock().unwrap();
            
            summary.total_matches += 1;
            
            // 记录匹配到的文件
            if matched_paths.insert(result.path.clone()) {
                summary.matched_files += 1;
                
                // 记录到日志
                if logger_clone.is_enabled() {
                    logger_clone.log_message(&format!("找到匹配: {}", result.path))?;
                }
            }
        }
        
        Ok(())
    });
    
    // 开始搜索
    println!("在 {} 中搜索: {}", search_path.display(), args.pattern);
    if let Some(min) = &args.min_size {
        println!("最小文件大小: {}", min);
    }
    if let Some(max) = &args.max_size {
        println!("最大文件大小: {}", max);
    }
    println!("使用正则表达式: {}", args.regex);
    println!("使用十六进制搜索: {}", args.hex);
    println!("并行搜索: {}", !args.no_parallel);
    println!("启用日志记录: {}", args.log);
    println!("遵循 .gitignore 规则: {}", config.search.respect_gitignore);
    println!("配置文件: {}", config_path.display());
    println!();

    // 记录搜索参数到日志
    if logger.is_enabled() {
        logger.log_message(&format!("搜索模式: {}", args.pattern))?;
        logger.log_message(&format!("目标目录: {}", search_path.display()))?;
        logger.log_message(&format!("使用正则表达式: {}", args.regex))?;
        logger.log_message(&format!("使用十六进制搜索: {}", args.hex))?;
        if let Some(min) = &args.min_size {
            logger.log_message(&format!("最小文件大小: {}", min))?;
        }
        if let Some(max) = &args.max_size {
            logger.log_message(&format!("最大文件大小: {}", max))?;
        }
        logger.log_message(&format!("并行搜索: {}", !args.no_parallel))?;
        logger.log_message(&format!("遵循 .gitignore 规则: {}", config.search.respect_gitignore))?;
    }
    
    // 执行文件遍历和搜索
    let tx_clone = tx.clone();
    let logger_clone = Arc::clone(&logger);
    let error_logger_clone = Arc::clone(&error_logger);
    let matcher_clone = matcher.clone();
    let cpu_monitor_clone = Arc::clone(&cpu_monitor);
    let context_lines = config.search.context_lines;

    let start_time = std::time::Instant::now();
    let (total_files, _) = domain::file_walker::scan_directory(
        &search_path,
        filter,
        !args.no_parallel,
        config.search.respect_gitignore,
        logger_clone,
        move |entry| {
            // 应用CPU性能控制
            cpu_monitor_clone.apply_throttle();

            // 在文件中搜索，捕获错误
            match domain::search::search_in_file(entry.path(), &matcher_clone, context_lines) {
                Ok(results) => {
                    // 发送结果
                    for result in results {
                        if tx_clone.send(result).is_err() {
                            break;
                        }
                    }
                }
                Err(err) => {
                    // 记录搜索错误到错误日志
                    let _ = error_logger_clone.log_error(
                        ErrorType::FileRead,
                        Some(&entry.path().to_string_lossy()),
                        "文件搜索失败",
                        Some(&err.to_string()),
                    );

                    // 不再向控制台输出错误，只记录到错误日志
                }
            }

            Ok(())
        },
    )?;
    
    // 关闭发送通道
    drop(tx);
    
    // 等待处理线程完成
    if let Err(err) = handle.join().unwrap() {
        eprintln!("处理结果时出错: {}", err);
    }
    
    // 更新最终统计信息
    let mut summary = summary.lock().unwrap();
    summary.total_files = total_files;
    
    // 计算总时间
    let duration = start_time.elapsed();
    
    // 停止CPU监控
    cpu_monitor.stop();

    // 完成错误日志记录
    error_logger.finalize()?;

    // 打印摘要
    summary.print()?;

    // 显示CPU监控状态
    let monitor_status = cpu_monitor.get_status();
    println!("性能监控: {}", monitor_status.format());

    // 显示错误摘要（如果有错误）
    error_logger.print_error_summary();

    // 完成调试日志记录
    if logger.is_enabled() {
        logger.finalize(total_files, summary.matched_files, summary.total_matches, duration)?;
        logger.log_message(&format!("最终CPU状态: {}", monitor_status.format()))?;
        logger.log_message(&format!("错误统计: {} 个错误", error_logger.get_total_errors()))?;
    }
    
    Ok(())
}
