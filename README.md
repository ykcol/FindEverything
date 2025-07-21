# FindEverything

A high-performance file content search tool for quickly finding text or binary content in directories.

**Language**: [English](README.md) | [中文](README_CN.md)

## ✨ Features

- 🚀 **High-Performance Search**: Built on ripgrep core libraries for lightning-fast results
- 🔍 **Multiple Search Modes**: Plain text, regular expressions, and hexadecimal value searches
- 📏 **Smart File Filtering**: Filter by file size, exclude directories, and respect .gitignore
- ⚡ **Parallel Processing**: Multi-threaded search utilizing all CPU cores
- 📊 **Performance Monitoring**: CPU usage monitoring with automatic throttling
- 📝 **Detailed Logging**: Optional comprehensive search logs with timestamps
- ⚙️ **Configurable Settings**: Customizable search behavior via config file

## 📦 Installation

### Option 1: Windows Installer (Recommended)
1. Download the latest installer: `FindEverything-0.3.0-setup.exe`
2. Run the installer as Administrator
3. Follow the installation wizard
4. The installer will automatically:
   - Install to `C:\Program Files\FindEverything`
   - Add to system PATH
   - Create Start Menu shortcuts
   - Generate default configuration file

### Option 2: Portable Version
1. Download and extract `FindEverything-0.3.0-release.zip`
2. Run `add_to_path.bat` as Administrator to add to PATH (optional)
3. Use directly from the extracted folder

## 🚀 Quick Start

```bash
# Basic search
FindEverything "search term"

# Search in specific directory
FindEverything "search term" C:\path\to\search

# Get help
FindEverything --help
```

## 📖 Usage

```
FindEverything [OPTIONS] <SEARCH_CONTENT> [DIRECTORY_PATH]
```

### Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `<SEARCH_CONTENT>` | Text content to search for | ✅ Yes |
| `[DIRECTORY_PATH]` | Directory to search in | ❌ No (defaults to current directory) |

### Options

| Option | Description | Example |
|--------|-------------|---------|
| `-r, --regex` | Use regular expression search | `--regex "hello.*world"` |
| `-x, --hex` | Parse search content as hexadecimal | `--hex "DEADBEEF"` |
| `--min-size <SIZE>` | Minimum file size filter | `--min-size 1M` |
| `--max-size <SIZE>` | Maximum file size filter | `--max-size 100M` |
| `--log` | Enable detailed logging | `--log` |
| `--respect-gitignore` | Respect .gitignore rules | `--respect-gitignore` |

## 💡 Examples

### Basic Text Search
```bash
# Search for "hello" in current directory
FindEverything hello

# Search in specific directory
FindEverything "error message" C:\logs
```

### Advanced Search
```bash
# Regular expression search
FindEverything --regex "hello.*world" C:\projects

# Hexadecimal search
FindEverything --hex "DEADBEEF" C:\binary_files

# Case-sensitive search with size filter
FindEverything --min-size 1M "API_KEY" C:\config
```

### Development Workflow
```bash
# Search code while respecting .gitignore
FindEverything --respect-gitignore "function.*main" C:\projects

# Debug with detailed logging
FindEverything --log "TODO" C:\source_code

# Search large files only
FindEverything --min-size 10M --max-size 1G "database" C:\data
```

## ⚙️ Configuration

FindEverything uses a `config.toml` file for customization:

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

## 🛠️ Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable version)
- Git

### Build Steps
```bash
git clone https://github.com/ykcol/FindEverything.git
cd FindEverything
cargo build --release
```

The compiled executable will be located at `target/release/FindEverything.exe`.

### Development
```bash
# Run tests
cargo test

# Run with debug output
cargo run -- --log "search term" ./

# Build installer (requires NSIS)
build_installer.bat
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE_NEW.txt) file for details.

## 🔗 Links

- **Repository**: [GitHub](https://github.com/ykcol/FindEverything)
- **Issues**: [Report bugs or request features](https://github.com/ykcol/FindEverything/issues)
- **Releases**: [Download latest version](https://github.com/ykcol/FindEverything/releases)