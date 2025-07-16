use std::path::Path;
// 移除未使用的导入
// use std::sync::Arc;

use anyhow::{Context, Result};
// 移除未使用的导入
// use bstr::ByteSlice;
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::{self, Searcher, SearcherBuilder, Sink, SinkMatch, SinkContext, SinkFinish};

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
}

/// 搜索结果接收器
pub struct ResultCollector {
    pub results: Vec<SearchResult>,
    pub matcher: RegexMatcher,
    // 存储当前正在处理的文件路径
    current_path: String,
}

impl ResultCollector {
    pub fn new(pattern: RegexMatcher) -> Self {
        Self {
            results: Vec::new(),
            matcher: pattern,
            current_path: String::new(),
        }
    }
    
    // 设置当前路径
    pub fn set_current_path(&mut self, path: &str) {
        self.current_path = path.to_string();
    }
}

impl Sink for ResultCollector {
    type Error = std::io::Error; // 使用标准的io::Error而不是anyhow::Error

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        let line = match std::str::from_utf8(mat.bytes()) {
            Ok(s) => s.to_string(),
            Err(_) => String::from_utf8_lossy(mat.bytes()).to_string(),
        };
        
        // 获取匹配的具体文本
        let matched_text = if let Some(m) = self.matcher.find(mat.bytes())? {
            String::from_utf8_lossy(&mat.bytes()[m.start()..m.end()]).to_string()
        } else {
            // 如果找不到匹配（应该不会发生），返回整行
            line.clone()
        };
        
        // 使用当前路径和行号
        let line_number = mat.line_number().unwrap_or(0);
        
        self.results.push(SearchResult {
            path: self.current_path.clone(),
            line_number,
            line,
            matched_text,
        });
        
        Ok(true)
    }
    
    fn context(&mut self, _searcher: &Searcher, _ctx: &SinkContext<'_>) -> Result<bool, Self::Error> {
        // 不处理上下文行
        Ok(true)
    }
    
    fn finish(&mut self, _searcher: &Searcher, _: &SinkFinish) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// 在单个文件中搜索
pub fn search_in_file(path: &Path, matcher: &RegexMatcher) -> Result<Vec<SearchResult>> {
    let mut collector = ResultCollector::new(matcher.clone());
    // 设置当前处理的文件路径
    collector.set_current_path(path.to_string_lossy().to_string().as_str());
    
    let mut searcher = SearcherBuilder::new()
        .binary_detection(grep_searcher::BinaryDetection::quit(b'\x00'))
        .line_number(true)
        .build();
    
    searcher.search_path(matcher, path, &mut collector)
        .with_context(|| format!("搜索文件失败: {}", path.display()))?;
    
    Ok(collector.results)
} 