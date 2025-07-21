// 新的三层架构模块
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// 重新导出主要类型
pub use domain::{SearchPattern, SearchResult, FileFilter};
pub use application::Config;
pub use infrastructure::{Logger, ErrorLogger, ErrorType, CpuMonitor};
pub use presentation::{SearchSummary, print_search_result};
