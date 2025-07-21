# FindEverything

高性能文件内容搜索工具，快速查找目录中的文本或二进制内容。

**语言**: [English](README.md) | [中文](README_CN.md)

## ✨ 功能特性

- 🚀 **高性能搜索**: 基于ripgrep核心库，搜索速度极快
- 🔍 **多种搜索模式**: 支持纯文本、正则表达式和十六进制值搜索
- 📏 **智能文件过滤**: 按文件大小过滤，排除目录，支持.gitignore规则
- ⚡ **并行处理**: 多线程搜索，充分利用所有CPU核心
- 📊 **性能监控**: CPU使用率监控，自动节流控制
- 📝 **详细日志**: 可选的详细搜索日志，包含时间戳
- ⚙️ **可配置设置**: 通过配置文件自定义搜索行为

## 📦 安装方式

### 方式一：Windows安装程序（推荐）
1. 下载最新安装程序：`FindEverything-0.3.0-setup.exe`
2. 以管理员身份运行安装程序
3. 按照安装向导完成安装
4. 安装程序将自动：
   - 安装到 `C:\Program Files\FindEverything`
   - 添加到系统PATH环境变量
   - 创建开始菜单快捷方式
   - 生成默认配置文件

### 方式二：便携版
1. 下载并解压 `FindEverything-0.3.0-release.zip`
2. 以管理员身份运行 `add_to_path.bat` 添加到PATH（可选）
3. 直接从解压目录使用

## 🚀 快速开始

```bash
# 基本搜索
FindEverything "搜索内容"

# 在指定目录搜索
FindEverything "搜索内容" C:\要搜索的路径

# 获取帮助
FindEverything --help
```

## 📖 使用说明

```
FindEverything [选项] <搜索内容> [目录路径]
```

### 参数说明

| 参数 | 描述 | 必需 |
|------|------|------|
| `<搜索内容>` | 要搜索的文本内容 | ✅ 是 |
| `[目录路径]` | 要搜索的目录 | ❌ 否（默认为当前目录） |

### 选项说明

| 选项 | 描述 | 示例 |
|------|------|------|
| `-r, --regex` | 使用正则表达式搜索 | `--regex "hello.*world"` |
| `-x, --hex` | 将搜索内容解析为十六进制 | `--hex "DEADBEEF"` |
| `--min-size <大小>` | 最小文件大小过滤 | `--min-size 1M` |
| `--max-size <大小>` | 最大文件大小过滤 | `--max-size 100M` |
| `--log` | 启用详细日志记录 | `--log` |
| `--respect-gitignore` | 遵循.gitignore规则 | `--respect-gitignore` |

## 💡 使用示例

### 基本文本搜索
```bash
# 在当前目录搜索"hello"
FindEverything hello

# 在指定目录搜索
FindEverything "错误信息" C:\日志文件
```

### 高级搜索
```bash
# 正则表达式搜索
FindEverything --regex "hello.*world" C:\项目目录

# 十六进制搜索
FindEverything --hex "DEADBEEF" C:\二进制文件

# 大小写敏感搜索并过滤文件大小
FindEverything --min-size 1M "API_KEY" C:\配置文件
```

### 开发工作流
```bash
# 搜索代码时遵循.gitignore规则
FindEverything --respect-gitignore "function.*main" C:\项目

# 启用详细日志进行调试
FindEverything --log "TODO" C:\源代码

# 只搜索大文件
FindEverything --min-size 10M --max-size 1G "数据库" C:\数据
```

## ⚙️ 配置文件

FindEverything使用 `config.toml` 文件进行自定义配置：

```toml
[search]
default_search_path = "."
context_lines = 5
respect_gitignore = false

[performance]
cpu_threshold = 80.0
search_delay_ms = 100

[exclude]
default_dirs = [".git", "node_modules", "target", ".vscode", ".idea"]
default_files = []

[display]
max_line_length = 200
highlight_matches = true
```

## 🛠️ 从源码构建

### 前置要求
- [Rust](https://rustup.rs/)（最新稳定版）
- Git

### 构建步骤
```bash
git clone https://github.com/ykcol/FindEverything.git
cd FindEverything
cargo build --release
```

编译后的可执行文件位于 `target/release/FindEverything.exe`。

### 开发
```bash
# 运行测试
cargo test

# 带调试输出运行
cargo run -- --log "搜索内容" ./

# 构建安装程序（需要NSIS）
build_installer.bat
```

## 🤝 贡献

欢迎贡献代码！请随时提交Pull Request。

## 📄 许可证

本项目采用MIT许可证 - 详见 [LICENSE](LICENSE_NEW.txt) 文件。

## 🔗 相关链接

- **代码仓库**: [GitHub](https://github.com/ykcol/FindEverything)
- **问题反馈**: [报告bug或请求功能](https://github.com/ykcol/FindEverything/issues)
- **版本发布**: [下载最新版本](https://github.com/ykcol/FindEverything/releases)
