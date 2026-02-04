# phpx

An npx-like tool for PHP, written in Rust.

`phpx` allows you to run PHP command-line tools (typically `.phar` files) without requiring global or local persistent installation. Through intelligent caching and version management, it ensures execution efficiency while maintaining a clean and isolated development environment.

## ✨ Features

- 🚀 **Zero Pollution** - Tools run without polluting global or project local environments
- 📦 **Smart Caching** - Automatic download and caching of tools, supporting offline usage
- 🔄 **Version Management** - Support for semantic version constraints and parallel version caching
- 🔒 **Security Verification** - File hash verification support (GPG signature verification in development)
- ⚡ **High Performance** - Asynchronous downloads with near-native tool startup speed
- 🛠️ **Multi-Source Support** - Support for Packagist, GitHub Releases, and direct URLs

## 📦 Installation

### Build from Source

```bash
# Clone the project
git clone https://github.com/pfinalcub/phpx.git
cd phpx

# Build the project
cargo build --release

# Install to system path (optional)
sudo cp target/release/phpx /usr/local/bin/
```

### System Requirements

- **Rust**: 1.70+ (for building)
- **PHP**: 7.4+ (for running PHP tools)
- **Operating Systems**: macOS, Linux, WSL2

## 🚀 Quick Start

### Basic Usage

```bash
# Run PHPStan for code analysis
phpx phpstan analyse src/

# Run PHP-CS-Fixer to format code
phpx php-cs-fixer fix /path/to/file.php

# Use specific tool versions
phpx phpstan@^1.10 analyse --level=max src/
phpx php-cs-fixer@^3.14 fix --dry-run

# View tool help
phpx php-cs-fixer --help
phpx php-cs-fixer fix --help
```

### Cache Management

```bash
# Clean cache for a specific tool
phpx cache clean phpstan

# Clean all cache
phpx cache clean

# List cached tools
phpx cache list

# View cache details for a tool
phpx cache info phpstan
```

## 📋 Command Line Options

### Global Options

```bash
# Force clear cache and re-download
phpx --clear-cache phpstan analyse src/

# Don't use cache for this execution
phpx --no-cache php-cs-fixer fix file.php

# Skip security verification
phpx --skip-verify phpstan analyse src/

# Specify PHP binary path
phpx --php /usr/local/bin/php8.1 phpstan analyse src/

# Ignore local project tools, use remote versions
phpx --no-local phpstan analyse src/

# Enable verbose logging
phpx --verbose phpstan analyse src/
```

### Subcommands

- `phpx cache clean [tool]` - Clean cache
- `phpx cache list` - List cached tools
- `phpx cache info <tool>` - View cache details
- `phpx config get <key>` - Get configuration (in development)
- `phpx config set <key> <value>` - Set configuration (in development)

## 🔧 How It Works

### Execution Flow

1. **Parse Tool Identifier** - Parse tool name and version constraints
2. **Check Local Tools** - Prioritize checking project `vendor/bin/` and global Composer directories
3. **Check Cache** - Look for tool versions in local cache
4. **Resolve Download Source** - Get tool information from Packagist, GitHub Releases, or direct URLs
5. **Download Tool** - Asynchronously download `.phar` file to cache directory
6. **Security Verification** - Verify file hash (GPG signature verification in development)
7. **Execute Tool** - Use system PHP to execute the downloaded tool

### Supported Source Types

- **Packagist**: `phpx phpstan`
- **GitHub Releases**: `phpx php-cs-fixer`
- **Direct URL**: Automatically infer common release patterns

## ⚙️ Configuration

### Configuration File Locations

- macOS/Linux: `~/.config/phpx/config.toml`
- Windows: `%APPDATA%/phpx/config.toml`

### Configuration Example

```toml
# Cache configuration
cache_dir = "~/.cache/phpx"
cache_ttl = 604800  # 7 days
max_cache_size = 1073741824  # 1GB

# Security configuration
skip_verify = false

# PHP configuration
default_php_path = "/usr/bin/php"

# Download mirrors
download_mirrors = [
    "https://packagist.org",
    "https://github.com",
]
```

## 🛠️ Development

### Project Structure

```
src/
├── main.rs          # Program entry point
├── lib.rs           # Module declarations
├── cli.rs           # Command-line interface
├── runner.rs        # Core execution flow
├── resolver.rs      # Tool resolver
├── download.rs      # File download
├── cache.rs         # Cache management
├── executor.rs      # PHP executor
├── config.rs        # Configuration management
├── security.rs      # Security verification
└── error.rs         # Error handling
```

### Building and Testing

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Code linting
cargo clippy

# Code formatting
cargo fmt
```

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Plan

- [x] **Phase 1**: Core functionality implementation (completed)
- [ ] **Phase 2**: Security verification and configuration system improvements
- [ ] **Phase 3**: Advanced features and user experience optimization

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [npx](https://github.com/npm/npx)
- Design concepts borrowed from [phive](https://github.com/phar-io/phive)

## 📞 Support

If you encounter issues or have suggestions, please:

1. Check [Issues](https://github.com/pfinalcub/phpx/issues)
2. Submit a new Issue
3. Or contact us via email

---

**phpx** - Making PHP tool execution simpler! 🚀