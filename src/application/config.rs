use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 搜索相关配置
    pub search: SearchConfig,
    /// 性能相关配置
    pub performance: PerformanceConfig,
    /// 排除规则配置
    pub exclude: ExcludeConfig,
    /// 显示相关配置
    pub display: DisplayConfig,
}

/// 搜索配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// 默认搜索路径
    pub default_search_path: String,
    /// 匹配结果显示的前后行数
    pub context_lines: usize,
    /// 是否遵循 .gitignore 规则
    pub respect_gitignore: bool,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// CPU使用率阈值百分比
    pub cpu_threshold: f32,
    /// 高CPU负载时的搜索延迟毫秒数
    pub search_delay_ms: u64,
}

/// 排除规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeConfig {
    /// 默认排除的目录
    pub default_dirs: Vec<String>,
    /// 默认排除的文件
    pub default_files: Vec<String>,
}

/// 显示配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// 最大行长度
    pub max_line_length: usize,
    /// 是否高亮匹配内容
    pub highlight_matches: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search: SearchConfig {
                default_search_path: ".".to_string(),
                context_lines: 5,
                respect_gitignore: false,
            },
            performance: PerformanceConfig {
                cpu_threshold: 80.0,
                search_delay_ms: 100,
            },
            exclude: ExcludeConfig {
                default_dirs: vec![
                    ".git".to_string(),
                    "node_modules".to_string(),
                    "target".to_string(),
                    ".vscode".to_string(),
                    ".idea".to_string(),
                ],
                default_files: vec![],
            },
            display: DisplayConfig {
                max_line_length: 200,
                highlight_matches: true,
            },
        }
    }
}

impl Config {
    /// 从配置文件加载配置，如果文件不存在则创建默认配置文件
    pub fn load_or_create(config_path: &Path) -> Result<Self> {
        if config_path.exists() {
            Self::load_from_file(config_path)
        } else {
            let config = Self::default();
            config.save_to_file(config_path)?;
            println!("已创建默认配置文件: {}", config_path.display());
            Ok(config)
        }
    }

    /// 从文件加载配置
    pub fn load_from_file(config_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("无法读取配置文件: {}", config_path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("无法解析配置文件: {}", config_path.display()))?;
        
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, config_path: &Path) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .context("无法序列化配置")?;
        
        fs::write(config_path, content)
            .with_context(|| format!("无法写入配置文件: {}", config_path.display()))?;
        
        Ok(())
    }

    /// 获取配置文件的默认路径
    pub fn default_config_path() -> Result<PathBuf> {
        // 尝试获取程序所在目录
        let exe_path = std::env::current_exe()
            .context("无法获取程序路径")?;
        
        let exe_dir = exe_path.parent()
            .context("无法获取程序目录")?;
        
        Ok(exe_dir.join("config.toml"))
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        if self.search.context_lines > 50 {
            anyhow::bail!("context_lines 不能超过 50");
        }
        
        if self.performance.cpu_threshold < 10.0 || self.performance.cpu_threshold > 100.0 {
            anyhow::bail!("cpu_threshold 必须在 10-100 之间");
        }
        
        if self.performance.search_delay_ms > 10000 {
            anyhow::bail!("search_delay_ms 不能超过 10000");
        }
        
        if self.display.max_line_length < 50 {
            anyhow::bail!("max_line_length 不能小于 50");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.search.default_search_path, ".");
        assert_eq!(config.search.context_lines, 5);
        assert_eq!(config.performance.cpu_threshold, 80.0);
        assert!(config.exclude.default_dirs.contains(&".git".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.search.context_lines, deserialized.search.context_lines);
        assert_eq!(config.performance.cpu_threshold, deserialized.performance.cpu_threshold);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        // 测试保存和加载
        let original_config = Config::default();
        original_config.save_to_file(&config_path).unwrap();
        
        let loaded_config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(original_config.search.context_lines, loaded_config.search.context_lines);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // 测试有效配置
        assert!(config.validate().is_ok());
        
        // 测试无效的 context_lines
        config.search.context_lines = 100;
        assert!(config.validate().is_err());
        
        // 重置并测试无效的 cpu_threshold
        config = Config::default();
        config.performance.cpu_threshold = 150.0;
        assert!(config.validate().is_err());
    }
}
