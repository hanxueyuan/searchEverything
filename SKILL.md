# searchEverything Skill

**Lightning-fast local file search tool for LLMs**

## Description

searchEverything is a high-performance local file search CLI tool designed for LLM integration. It provides fast file search, indexing, and file management capabilities with LLM-friendly JSON output.

## Features

- 🔍 **Multiple Search Modes**: Glob wildcards, regex, fuzzy search
- ⚡ **High Performance**: 13,000 files/second indexing speed
- 🤖 **LLM-Friendly**: Structured JSON output with human-readable fields
- 📦 **OpenClaw Integration**: Deep integration with automatic installation
- 🌍 **Bilingual**: Full English and Chinese support

## Installation

```bash
# Via OpenClaw (recommended)
openclaw skills install searchEverything

# Manual installation
curl -L https://github.com/hanxueyuan/searchEverything/releases/latest/download/searchEverything-x86_64-unknown-linux-gnu -o searchEverything
chmod +x searchEverything
sudo mv searchEverything /usr/local/bin/
```

## Usage Examples

```bash
# Search for files
searchEverything search "*.pdf" --format json

# Search with regex
searchEverything search ".*\.rs$" --regex --format json

# Fuzzy search
searchEverything search "config" --fuzzy --format json

# View file info
searchEverything info /path/to/file

# Read file content
searchEverything cat /path/to/file

# Manage index
searchEverything index add /path/to/project
searchEverything index status
```

## OpenClaw Integration

This skill automatically installs the searchEverything binary when installed via OpenClaw. No manual setup required.

### Natural Language Triggers

- "Search for PDF files"
- "Find all Rust source code"
- "Show me the config files"
- "Read the README file"
- "搜索 PDF 文件" (Chinese)
- "查找代码文件" (Chinese)

## Configuration

Config file location: `~/.openclaw/skills/searchEverything/config.yaml`

## License

MIT License - See [LICENSE](../LICENSE) for details.

## Repository

https://github.com/hanxueyuan/searchEverything
