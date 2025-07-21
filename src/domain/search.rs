use std::path::Path;

use anyhow::{Context, Result};
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;

/// 搜索模式类型
#[derive(Debug, Clone)]
pub enum SearchPattern {
    /// 普通文本搜索
    Text(String),
    /// 十六进制值搜索
    Hex(Vec<u8>),
    /// 正则表达式搜索
    Regex(String),
}

impl SearchPattern {
    /// 从输入字符串创建搜索模式
    pub fn from_input(input: &str, is_regex: bool, is_hex: bool) -> Result<Self> {
        if is_regex {
            // 验证正则表达式有效性
            let _ = regex::Regex::new(input)
                .context("无效的正则表达式")?;
            Ok(SearchPattern::Regex(input.to_string()))
        } else if is_hex {
            // 解析十六进制字符串
            let hex_bytes = hex::decode(input.replace(' ', ""))
                .context("无效的十六进制值")?;
            Ok(SearchPattern::Hex(hex_bytes))
        } else {
            Ok(SearchPattern::Text(input.to_string()))
        }
    }

    /// 获取匹配器
    pub fn get_matcher(&self) -> Result<RegexMatcher> {
        match self {
            SearchPattern::Text(text) => {
                // 转义正则表达式特殊字符
                let escaped = regex::escape(text);
                RegexMatcher::new(&escaped)
                    .context("无法创建文本匹配器")
            }
            SearchPattern::Hex(bytes) => {
                // 将十六进制字节转换为正则表达式
                let pattern = bytes.iter()
                    .map(|b| format!(r"\x{:02x}", b))
                    .collect::<String>();
                RegexMatcher::new(&pattern)
                    .context("无法创建十六进制匹配器")
            }
            SearchPattern::Regex(pattern) => {
                RegexMatcher::new(pattern)
                    .context("无法创建正则表达式匹配器")
            }
        }
    }
}

/// 搜索结果
#[derive(Debug)]
pub struct SearchResult {
    pub path: String,
    pub line_number: u64,
    pub line: String,
    pub matched_text: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
}

/// 在单个文件中搜索
pub fn search_in_file(path: &Path, matcher: &RegexMatcher, context_lines: usize) -> Result<Vec<SearchResult>> {
    // 读取文件内容
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("无法读取文件: {}", path.display()))?;

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut results = Vec::new();

    // 查找匹配行
    for (line_idx, line) in lines.iter().enumerate() {
        if let Ok(Some(m)) = matcher.find(line.as_bytes()) {
            let matched_text = String::from_utf8_lossy(&line.as_bytes()[m.start()..m.end()]).to_string();
            
            // 获取上下文行
            let context_before = get_context_lines(&lines, line_idx, context_lines, true);
            let context_after = get_context_lines(&lines, line_idx, context_lines, false);
            
            results.push(SearchResult {
                path: path.to_string_lossy().to_string(),
                line_number: (line_idx + 1) as u64, // 转换为1基索引
                line: line.clone(),
                matched_text,
                context_before,
                context_after,
            });
        }
    }

    Ok(results)
}

/// 获取上下文行
fn get_context_lines(lines: &[String], line_idx: usize, context_lines: usize, before: bool) -> Vec<String> {
    if before {
        let start = if line_idx >= context_lines {
            line_idx - context_lines
        } else {
            0
        };
        lines[start..line_idx].to_vec()
    } else {
        let end = std::cmp::min(line_idx + 1 + context_lines, lines.len());
        lines[line_idx + 1..end].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_pattern_text() {
        let pattern = SearchPattern::from_input("hello", false, false).unwrap();
        match pattern {
            SearchPattern::Text(text) => assert_eq!(text, "hello"),
            _ => panic!("Expected Text pattern"),
        }
    }

    #[test]
    fn test_search_pattern_regex() {
        let pattern = SearchPattern::from_input("h.*o", true, false).unwrap();
        match pattern {
            SearchPattern::Regex(regex) => assert_eq!(regex, "h.*o"),
            _ => panic!("Expected Regex pattern"),
        }
    }

    #[test]
    fn test_search_pattern_hex() {
        let pattern = SearchPattern::from_input("48656c6c6f", false, true).unwrap();
        match pattern {
            SearchPattern::Hex(bytes) => assert_eq!(bytes, vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]),
            _ => panic!("Expected Hex pattern"),
        }
    }

    #[test]
    fn test_get_matcher() {
        let pattern = SearchPattern::Text("test".to_string());
        let matcher = pattern.get_matcher().unwrap();
        
        // 测试匹配器是否能正确匹配
        let test_line = "this is a test line";
        assert!(matcher.find(test_line.as_bytes()).unwrap().is_some());
    }
}
