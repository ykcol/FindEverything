# FindEverything

一个高效的文件内容搜索工具，可以快速在目录中搜索指定的文本或二进制内容。

[English](README.md) | [中文](README_CN.md)

## 功能特点

- 高效搜索：使用 ripgrep 核心库实现高性能搜索
- 多种搜索模式：支持普通文本、正则表达式和十六进制值搜索
- 文件大小过滤：可设置最大/最小文件大小进行搜索
- 并行处理：多线程搜索加速
- 详细日志：可选生成详细的搜索日志
- 灵活的 .gitignore 处理：默认搜索所有文件，可选择遵循 .gitignore 规则

## 安装

1. 下载最新的安装包 `FindEverything-0.1.0-setup.exe`
2. 运行安装包，按照提示完成安装
3. 安装程序会自动将 FindEverything 添加到系统环境变量中，可以在任何位置使用

## 使用方法

```
FindEverything [选项] <搜索内容> [目录路径]
```

### 参数说明

- `<搜索内容>`: 要搜索的文本内容（必需）
- `[目录路径]`: 要搜索的目录（默认为当前目录）

### 选项

- `-r, --regex`: 使用正则表达式搜索
- `-x, --hex`: 将搜索内容解析为十六进制值
- `--min-size <大小>`: 最小文件大小 (例如 "1K", "1M", "1G")
- `--max-size <大小>`: 最大文件大小 (例如 "1K", "1M", "1G")
- `--no-parallel`: 不使用并行处理 (默认使用所有可用CPU)
- `--log`: 启用详细日志记录，日志文件将保存到程序同级目录下
- `--respect-gitignore`: 遵循 .gitignore 规则 (默认不遵循)

### 示例

搜索当前目录下所有包含 "hello" 的文件：
```
FindEverything hello
```

使用正则表达式搜索：
```
FindEverything --regex "hello.*world" C:\path\to\search
```

搜索十六进制值：
```
FindEverything --hex "DEADBEEF" C:\path\to\search
```

搜索大于 1MB 的文件：
```
FindEverything --min-size 1M "important data" C:\path\to\search
```

启用详细日志：
```
FindEverything --log "debug info" C:\path\to\search
```

遵循 .gitignore 规则：
```
FindEverything --respect-gitignore "code" C:\projects
```

## 构建

要从源代码构建，请确保已安装 Rust：

```
git clone https://github.com/ykcol/FindEverything.git
cd FindEverything
cargo build --release
```

编译后的可执行文件位于 `target/release/FindEverything.exe`。

## 许可

本项目采用 MIT 许可证。 