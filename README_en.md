# searchEverything

[中文](README.md) | [English](README_en.md)

---

Cross-platform file search CLI tool with OpenClaw integration.

## Features

- 🔍 **Three Search Modes**: Wildcard (default), Regular Expression, Fuzzy Search
- ⚡ **High Performance**: Rust implementation, Trie tree indexing, O(m) prefix search
- 🖥️ **Cross-Platform**: Windows/Linux/macOS
- 🤖 **OpenClaw Integration**: JSON output, LLM-friendly
- 📁 **Index Management**: Manually add/remove index directories
- 🔔 **Real-time Monitoring**: Linux inotify / Windows USN Journal
- 📊 **Streaming Output**: Real-time results with pagination support
- 📝 **Audit Logging**: Operation history tracking with query and export

## Installation

```bash
# Build from source
cargo build --release

# Copy binary to PATH
cp target/release/searchEverything /usr/local/bin/

# Verify installation
searchEverything --version
```

### Platform-Specific Installation

#### Linux

```bash
# Build and install
cargo build --release
sudo cp target/release/searchEverything /usr/local/bin/

# Or add to user PATH
cp target/release/searchEverything ~/.local/bin/
```

#### macOS

```bash
# Build and install
cargo build --release
cp target/release/searchEverything /usr/local/bin/

# Or via Homebrew (coming soon)
# brew install searchEverything
```

#### Windows

```powershell
# Build and install
cargo build --release
copy target\release\searchEverything.exe C:\Windows\System32\

# Or add to user PATH
copy target\release\searchEverything.exe %USERPROFILE%\.local\bin\
```

## Basic Usage

### Search Files

```bash
# Wildcard mode (default)
searchEverything search "*.pdf" --path D:/work
searchEverything search "report*"

# Regular expression mode
searchEverything search --regex ".*\.pdf$"
searchEverything search --regex "report-\d{4}\.docx"

# Fuzzy search
searchEverything search --fuzzy "repot"  # matches report
searchEverything search --fuzzy "readme"  # matches README.md

# JSON output (OpenClaw integration, default)
searchEverything search "*.md" --format json
```

### File Operations

```bash
# View file information
searchEverything info README.md
searchEverything info D:/work/report.pdf --json

# Read file content
searchEverything cat config.yaml
searchEverything cat app.log --lines 50  # First 50 lines
searchEverything cat app.log --tail --lines 10  # Last 10 lines

# Copy/Move/Delete
searchEverything copy "*.txt" /backup/
searchEverything move temp.zip /archive/
searchEverything delete "*.tmp" --force
```

### Index Management

```bash
# Index status (auto-indexes entire system on first launch)
searchEverything index status

# Exclude specific directories (privacy/system directories)
searchEverything index exclude add --path C:/Windows
searchEverything index exclude add --path /proc

# View exclusion list
searchEverything index exclude list

# Remove exclusion
searchEverything index exclude remove --path C:/Windows

# Add additional index paths
searchEverything index add --path /mnt/data

# Remove index path
searchEverything index remove --path /mnt/data

# Rebuild index (full)
searchEverything index rebuild

# Rebuild specific path
searchEverything index rebuild --path D:/work

# Quick status view
searchEverything index-status

# Real-time monitoring (Linux inotify / Windows USN)
searchEverything index watch /home    # Linux
searchEverything index watch C:/work  # Windows
```

### Streaming Output

```bash
# Streaming output (real-time results)
searchEverything search "*.log" --stream

# Pagination (20 results per page)
searchEverything search "*.txt" --stream --page-size 20

# Progress display automatically enabled for large file searches
searchEverything search "*.pdf" --stream
```

### Audit Logging

```bash
# View recent logs
searchEverything audit list
searchEverything audit list --limit 50
searchEverything audit list --command search

# Search logs
searchEverything audit search --command search --after 2024-01-01T00:00:00+08:00
searchEverything audit search --result error

# Export logs
searchEverything audit export --format json --output audit.json
searchEverything audit export --format csv --output audit.csv

# View statistics
searchEverything audit stats

# Clean old logs
searchEverything audit cleanup --days 30
```

## OpenClaw Skill Integration

### Install Skill

```bash
# Copy Skill definition
cp skills/openclaw-skill.yaml ~/.openclaw/skills/

# Register skill
openclaw skills register searchEverything
```

### Usage Examples

```
User: Search for PDF files in work directory
→ searchEverything search "*.pdf" --path D:/work --format json

User: Search 2024 reports with regex
→ searchEverything search --regex "report-2024.*\.docx"

User: Fuzzy find readme files
→ searchEverything search --fuzzy "readme"

User: View information about this file
→ searchEverything info D:/work/report.pdf

User: Read yesterday's meeting notes
→ searchEverything cat meeting-notes.md --lines 50

User: Copy these files to backup directory
→ searchEverything copy "D:/work/*.txt" "D:/backup/"
```

## Output Format

### JSON Output (Default, LLM-Friendly)

```json
{
  "results": [
    {
      "path": "/home/user/document.pdf",
      "size": 1048576,
      "size_human": "1.0 MB",
      "modified": "2026-03-10T14:30:00+00:00",
      "modified_human": "2 days ago",
      "file_type": "pdf",
      "is_dir": false
    }
  ],
  "total": 1,
  "scanned": 15234,
  "search_time_ms": 45,
  "summary": {
    "total_size": 1048576,
    "total_size_human": "1.0 MB",
    "file_types": "pdf: 1",
    "oldest_file": "/home/user/document.pdf",
    "newest_file": "/home/user/document.pdf"
  },
  "error": null
}
```

### Error Response

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "NOT_FOUND",
    "message": "File not found",
    "details": "Path /not/exist does not exist",
    "suggestion": "Please check if the path is correct, or use searchEverything search to find files"
  }
}
```

## Command Reference

| Command | Description | Example |
|---------|-------------|---------|
| `searchEverything search <pattern>` | Search for files | `searchEverything search "*.pdf" --path D:/work` |
| `searchEverything search --regex <pattern>` | Regex search | `searchEverything search --regex ".*\.pdf$"` |
| `searchEverything search --fuzzy <pattern>` | Fuzzy search | `searchEverything search --fuzzy "repot"` |
| `searchEverything info <file>` | View file info | `searchEverything info README.md` |
| `searchEverything cat <file>` | Read file content | `searchEverything cat config.yaml --lines 50` |
| `searchEverything copy <src> <dest>` | Copy files | `searchEverything copy "*.txt" /backup/` |
| `searchEverything move <src> <dest>` | Move files | `searchEverything move temp.zip /archive/` |
| `searchEverything delete <path>` | Delete files | `searchEverything delete "*.tmp" --force` |
| `searchEverything index add` | Add index directory | `searchEverything index add --path D:/work` |
| `searchEverything index remove` | Remove index directory | `searchEverything index remove --path D:/work` |
| `searchEverything index list` | List index directories | `searchEverything index list` |
| `searchEverything index rebuild` | Rebuild index | `searchEverything index rebuild --path D:/` |

## Configuration

### Configuration File Location

- **Linux/macOS**: `~/.config/searchEverything/config.yaml`
- **Windows**: `%APPDATA%\searchEverything\config.yaml`

### Example Configuration

```yaml
# searchEverything configuration

search:
  default_mode: wildcard  # wildcard, regex, or fuzzy
  default_limit: 100
  case_sensitive: false

output:
  default_format: json  # json or text
  pretty_print: true

index:
  auto_index: true
  watch_enabled: true
  exclude_patterns:
    - "**/node_modules/**"
    - "**/.git/**"
    - "**/__pycache__/**"

file_operations:
  delete_confirm: true
  backup_on_delete: false

advanced:
  debug: false
  log_level: info
```

## OpenClaw Integration

searchEverything provides deep integration with OpenClaw through a Skill definition.

### Features

- **Natural Language Triggers**: Automatically invoked based on user intent
- **JSON Output**: Structured data for LLM processing
- **Automatic Installation**: Skill includes install/update/uninstall hooks
- **Audit Logging**: All operations are logged for compliance

### Trigger Patterns

The skill responds to various natural language inputs:

- "Search for PDF files" → `search` command
- "Find all markdown files" → `search` command
- "View file information" → `info` command
- "Read this file" → `cat` command
- "Copy files to backup" → `copy` command
- "Delete temporary files" → `delete` command
- "Add to index" → `index_add` command

## Technical Stack

- **Language**: Rust
- **CLI Parsing**: clap
- **Serialization**: serde + serde_json
- **File Search**: walkdir + glob + regex
- **Cross-Platform**: Windows USN / Linux inotify

## Development

```bash
# Development mode
cargo run -- search "*.rs"

# Release build
cargo build --release

# Run tests
cargo test

# Code formatting
cargo fmt

# Code linting
cargo clippy
```

## Project Structure

```
searchEverything/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point
│   ├── commands/        # Command implementations
│   ├── error.rs         # Error definitions
│   └── output.rs        # Output formatting
├── skills/
│   └── openclaw-skill.yaml
└── docs/
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to get started.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Lint code: `cargo clippy`
7. Commit and push
8. Open a Pull Request

## License

MIT

---

**GitHub**: [hanxueyuan/searchEverything](https://github.com/hanxueyuan/searchEverything)  
**Author**: 韩雪原 (xueyuan_han@163.com)
