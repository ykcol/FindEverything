use std::io::{self, Write};
use std::time::Instant;

use anyhow::Result;

use crate::domain::search::SearchResult;

/// 格式化持续时间
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}.{:03}s", secs, duration.subsec_millis())
    }
}

/// 输出搜索结果
pub fn print_search_result(result: &SearchResult) -> Result<()> {
    let mut stdout = io::stdout().lock();

    // 输出文件路径和行号
    writeln!(stdout, "\x1b[1;32m{}\x1b[0m:\x1b[1;34m{}\x1b[0m", result.path, result.line_number)?;

    // 输出上下文行（之前）
    for (i, context_line) in result.context_before.iter().enumerate() {
        let line_num = result.line_number - (result.context_before.len() - i) as u64;
        writeln!(stdout, "\x1b[2;37m{:>6}:\x1b[0m  {}", line_num, context_line)?;
    }

    // 输出匹配行内容，高亮匹配部分
    let line = &result.line;
    let matched_text = &result.matched_text;

    write!(stdout, "\x1b[1;34m{:>6}:\x1b[0m  ", result.line_number)?;
    if let Some(idx) = line.find(matched_text) {
        let before = &line[..idx];
        let after = &line[idx + matched_text.len()..];
        
        write!(stdout, "{}", before)?;
        write!(stdout, "\x1b[1;31m{}\x1b[0m", matched_text)?;
        writeln!(stdout, "{}", after)?;
    } else {
        writeln!(stdout, "{}", line)?;
    }

    // 输出上下文行（之后）
    for (i, context_line) in result.context_after.iter().enumerate() {
        let line_num = result.line_number + (i + 1) as u64;
        writeln!(stdout, "\x1b[2;37m{:>6}:\x1b[0m  {}", line_num, context_line)?;
    }

    // 如果有上下文行，添加分隔符
    if !result.context_before.is_empty() || !result.context_after.is_empty() {
        writeln!(stdout, "\x1b[2;37m--\x1b[0m")?;
    }

    Ok(())
}

/// 搜索摘要
pub struct SearchSummary {
    pub start_time: Instant,
    pub total_files: u64,
    pub matched_files: u64,
    pub total_matches: u64,
}

impl SearchSummary {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            total_files: 0,
            matched_files: 0,
            total_matches: 0,
        }
    }

    pub fn print(&self) -> Result<()> {
        let duration = self.start_time.elapsed();
        
        println!("\n搜索摘要:");
        println!("----------------------------");
        println!("总用时: {}", format_duration(duration));
        println!("扫描文件: {}", self.total_files);
        println!("匹配文件: {}", self.matched_files);
        println!("匹配项数: {}", self.total_matches);
        
        Ok(())
    }
}
