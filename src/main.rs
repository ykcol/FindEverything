use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Parser;
use crossbeam_channel::bounded;

mod display;
mod logger;
mod searcher;
mod walker;

use display::SearchSummary;
use logger::Logger;
use searcher::{SearchPattern, SearchResult};
use walker::FileSizeFilter;

/// 查找文件内容的命令行工具
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// 要搜索的内容
    #[clap(required = true)]
    pattern: String,
    
    /// 要搜索的目录路径
    #[clap(default_value = ".")]
    path: PathBuf,
    
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
    
    /// 遵循 .gitignore 规则，默认情况下会搜索所有文件
    #[clap(long)]
    respect_gitignore: bool,
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
    
    // 初始化日志记录器
    let logger = Arc::new(Logger::new(args.log)?);
    
    // 解析搜索模式
    let pattern = SearchPattern::from_input(&args.pattern, args.regex, args.hex)?;
    let matcher = pattern.get_matcher()?;
    
    // 解析文件大小过滤器
    let filter = FileSizeFilter {
        min_size: args.min_size.as_deref().map(parse_size).transpose()?,
        max_size: args.max_size.as_deref().map(parse_size).transpose()?,
    };
    
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
            display::print_search_result(&result)?;
            
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
    println!("在 {} 中搜索: {}", args.path.display(), args.pattern);
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
    println!("遵循 .gitignore 规则: {}", args.respect_gitignore);
    println!();
    
    // 记录搜索参数到日志
    if logger.is_enabled() {
        logger.log_message(&format!("搜索模式: {}", args.pattern))?;
        logger.log_message(&format!("目标目录: {}", args.path.display()))?;
        logger.log_message(&format!("使用正则表达式: {}", args.regex))?;
        logger.log_message(&format!("使用十六进制搜索: {}", args.hex))?;
        if let Some(min) = &args.min_size {
            logger.log_message(&format!("最小文件大小: {}", min))?;
        }
        if let Some(max) = &args.max_size {
            logger.log_message(&format!("最大文件大小: {}", max))?;
        }
        logger.log_message(&format!("并行搜索: {}", !args.no_parallel))?;
        logger.log_message(&format!("遵循 .gitignore 规则: {}", args.respect_gitignore))?;
    }
    
    // 执行文件遍历和搜索
    let tx_clone = tx.clone();
    let logger_clone = Arc::clone(&logger);
    let matcher_clone = matcher.clone();
    
    let start_time = std::time::Instant::now();
    let (total_files, _) = walker::scan_directory(
        &args.path,
        filter,
        !args.no_parallel,
        args.respect_gitignore,
        logger_clone,
        move |entry| {
            // 在文件中搜索
            let results = searcher::search_in_file(entry.path(), &matcher_clone)?;
            
            // 发送结果
            for result in results {
                if tx_clone.send(result).is_err() {
                    break;
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
    
    // 打印摘要
    summary.print()?;
    
    // 完成日志记录
    if logger.is_enabled() {
        logger.finalize(total_files, summary.matched_files, summary.total_matches, duration)?;
    }
    
    Ok(())
}
