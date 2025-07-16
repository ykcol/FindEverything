# FindEverything

A powerful file content search tool that quickly searches for specified text or binary content in directories.

[English](README.md) | [中文](README_CN.md)

## Features

- High-performance search: Powered by ripgrep core libraries
- Multiple search modes: Support for plain text, regular expressions, and hexadecimal value searches
- File size filtering: Set minimum/maximum file size for search
- Parallel processing: Multi-threaded search for faster results
- Detailed logging: Optional generation of detailed search logs
- Flexible .gitignore handling: Searches all files by default, with option to respect .gitignore rules

## Installation

1. Download the latest installer `FindEverything-0.1.0-setup.exe`
2. Run the installer and follow the prompts to complete installation
3. The installer will automatically add FindEverything to your system PATH, making it available from anywhere

## Usage

```
FindEverything [OPTIONS] <SEARCH_CONTENT> [DIRECTORY_PATH]
```

### Parameters

- `<SEARCH_CONTENT>`: The text content to search for (required)
- `[DIRECTORY_PATH]`: The directory to search in (defaults to current directory)

### Options

- `-r, --regex`: Use regular expression search
- `-x, --hex`: Parse search content as hexadecimal value
- `--min-size <SIZE>`: Minimum file size (e.g., "1K", "1M", "1G")
- `--max-size <SIZE>`: Maximum file size (e.g., "1K", "1M", "1G")
- `--no-parallel`: Disable parallel processing (uses all available CPUs by default)
- `--log`: Enable detailed logging, log files will be saved in the same directory as the program
- `--respect-gitignore`: Respect .gitignore rules (ignored by default)

### Examples

Search for all files containing "hello" in the current directory:
```
FindEverything hello
```

Search using regular expressions:
```
FindEverything --regex "hello.*world" C:\path\to\search
```

Search for hexadecimal values:
```
FindEverything --hex "DEADBEEF" C:\path\to\search
```

Search files larger than 1MB:
```
FindEverything --min-size 1M "important data" C:\path\to\search
```

Enable detailed logging:
```
FindEverything --log "debug info" C:\path\to\search
```

Respect .gitignore rules:
```
FindEverything --respect-gitignore "code" C:\projects
```

## Building from Source

To build from source, ensure you have Rust installed:

```
git clone https://github.com/ykcol/FindEverything.git
cd FindEverything
cargo build --release
```

The compiled executable will be located at `target/release/FindEverything.exe`.

## License

This project is licensed under the MIT License. 