# phpx

一个类似 npx 的 PHP 工具运行器，用 Rust 编写。

`phpx` 允许您直接运行 PHP 命令行工具（通常是 `.phar` 文件），而无需进行全局或本地的持久化安装。通过智能缓存和版本管理，在保证执行效率的同时，确保开发环境的绝对干净与隔离。

## ✨ 特性

- 🚀 **零污染运行** - 工具运行不污染全局或项目本地环境
- 📦 **智能缓存** - 自动下载并缓存工具，支持离线使用
- 🔄 **版本管理** - 支持语义化版本约束和并行版本缓存
- 🔒 **安全验证** - 支持文件哈希验证（GPG 签名验证开发中）
- ⚡ **高性能** - 异步下载，接近本地工具的启动速度
- 🛠️ **多源支持** - 支持 Packagist、GitHub Releases 和直接 URL

## 📦 安装

### 从源码构建

```bash
# 克隆项目
git clone https://github.com/your-username/phpx.git
cd phpx

# 构建项目
cargo build --release

# 安装到系统路径（可选）
sudo cp target/release/phpx /usr/local/bin/
```

### 系统要求

- **Rust**: 1.70+ (用于构建)
- **PHP**: 7.4+ (用于运行 PHP 工具)
- **操作系统**: macOS, Linux, WSL2

## 🚀 快速开始

### 基本用法

```bash
# 运行 PHPStan 进行代码分析
phpx phpstan analyse src/

# 运行 PHP-CS-Fixer 格式化代码
phpx php-cs-fixer fix /path/to/file.php

# 使用特定版本的工具
phpx phpstan@^1.10 analyse --level=max src/
phpx php-cs-fixer@^3.14 fix --dry-run

# 查看工具帮助
phpx php-cs-fixer --help
phpx php-cs-fixer fix --help
```

### 缓存管理

```bash
# 清理指定工具的缓存
phpx cache clean phpstan

# 清理所有缓存
phpx cache clean

# 查看已缓存的工具
phpx cache list

# 查看工具缓存详情
phpx cache info phpstan
```

## 📋 命令行选项

### 全局选项

```bash
# 强制清除缓存并重新下载
phpx --clear-cache phpstan analyse src/

# 本次执行不使用缓存
phpx --no-cache php-cs-fixer fix file.php

# 跳过安全验证
phpx --skip-verify phpstan analyse src/

# 指定 PHP 二进制路径
phpx --php /usr/local/bin/php8.1 phpstan analyse src/

# 忽略项目本地工具，使用远程版本
phpx --no-local phpstan analyse src/

# 启用详细日志
phpx --verbose phpstan analyse src/
```

### 子命令

- `phpx cache clean [tool]` - 清理缓存
- `phpx cache list` - 列出缓存
- `phpx cache info <tool>` - 查看缓存详情
- `phpx config get <key>` - 获取配置（开发中）
- `phpx config set <key> <value>` - 设置配置（开发中）

## 🔧 工作原理

### 执行流程

1. **解析工具标识符** - 解析工具名和版本约束
2. **检查本地工具** - 优先检查项目 `vendor/bin/` 和全局 Composer 目录
3. **检查缓存** - 查找本地缓存中的工具版本
4. **解析下载源** - 从 Packagist、GitHub Releases 或直接 URL 获取工具信息
5. **下载工具** - 异步下载 `.phar` 文件到缓存目录
6. **安全验证** - 验证文件哈希（GPG 签名验证开发中）
7. **执行工具** - 使用系统 PHP 执行下载的工具

### 支持的源类型

- **Packagist**: `phpx phpstan`
- **GitHub Releases**: `phpx php-cs-fixer`
- **直接 URL**: 自动推断常见发布模式

## ⚙️ 配置

### 配置文件位置

- macOS/Linux: `~/.config/phpx/config.toml`
- Windows: `%APPDATA%/phpx/config.toml`

### 配置示例

```toml
# 缓存配置
cache_dir = "~/.cache/phpx"
cache_ttl = 604800  # 7天
max_cache_size = 1073741824  # 1GB

# 安全配置
skip_verify = false

# PHP 配置
default_php_path = "/usr/bin/php"

# 下载镜像源
download_mirrors = [
    "https://packagist.org",
    "https://github.com",
]
```

## 🛠️ 开发

### 项目结构

```
src/
├── main.rs          # 程序入口点
├── lib.rs           # 模块声明
├── cli.rs           # 命令行接口
├── runner.rs        # 核心执行流程
├── resolver.rs      # 工具解析器
├── download.rs      # 文件下载
├── cache.rs         # 缓存管理
├── executor.rs      # PHP 执行器
├── config.rs        # 配置管理
├── security.rs      # 安全验证
└── error.rs         # 错误处理
```

### 构建和测试

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化代码
cargo fmt
```

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

### 开发计划

- [x] **Phase 1**: 核心功能实现（已完成）
- [ ] **Phase 2**: 安全验证和配置系统完善
- [ ] **Phase 3**: 高级功能和用户体验优化

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- 灵感来源于 [npx](https://github.com/npm/npx)
- 借鉴了 [phive](https://github.com/phar-io/phive) 的设计理念

## 📞 支持

如果您遇到问题或有建议，请：

1. 查看 [Issues](https://github.com/your-username/phpx/issues)
2. 提交新的 Issue
3. 或通过邮件联系我们

---

**phpx** - 让 PHP 工具运行更简单！ 🚀