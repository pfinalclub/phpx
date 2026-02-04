# Contributing Guide

Thank you for your interest in the phpx project! We welcome contributions of all kinds.

## 🚀 Quick Start

### Development Environment Setup

1. **Clone the Project**
   ```bash
   git clone https://github.com/pfinalcub/phpx.git
   cd phpx
   ```

2. **Install Rust Toolchain**
   - Install [Rust](https://www.rust-lang.org/tools/install) 1.70+
   - Install necessary tools:
     ```bash
     rustup component add clippy
     rustup component add rustfmt
     ```

3. **Build the Project**
   ```bash
   cargo build
   ```

## 🎯 Development Process

### Code Style

- Follow official Rust code style
- Use `cargo fmt` for code formatting
- Use `cargo clippy` for code linting
- Function naming uses snake_case
- Maximum 80 characters per line

### Commit Message Convention

Use conventional commit format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test-related changes
- `chore`: Build process or tool changes

### Branch Strategy

- `main`: Main branch, stable releases
- `develop`: Development branch
- `feature/*`: Feature branches
- `fix/*`: Bug fix branches

## 🐛 Reporting Issues

Before reporting an issue, please:

1. Check if a related issue already exists
2. Provide detailed reproduction steps
3. Include error logs and system information
4. If possible, provide minimal reproducible code

## 🔧 Development Tasks

### Current Development Focus

#### Phase 2: Security Verification and Configuration System Improvements
- [ ] Implement GPG/PGP signature verification
- [ ] Improve configuration file read/write functionality
- [ ] Implement `phpx config` subcommands
- [ ] Implement `phpx self-update` command

#### Phase 3: Advanced Features and User Experience Optimization
- [ ] Add progress bar display
- [ ] Implement cache TTL and size limits
- [ ] Support HTTP proxy
- [ ] Add auto-completion support

### How to Start Contributing

1. **Choose a Task**: Select from the task list above
2. **Create a Branch**: `git checkout -b feature/your-feature`
3. **Implement Feature**: Write code and tests
4. **Run Tests**: `cargo test`
5. **Code Quality**: `cargo clippy && cargo fmt`
6. **Submit PR**: Describe feature changes and test results

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test test_name

# Run integration tests
cargo test --test integration
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage tests
cargo tarpaulin --ignore-tests
```

## 📚 Documentation

### Code Documentation

- All public functions and complex logic should have comments
- Use Rustdoc format for documentation comments
- Run `cargo doc` to generate documentation

### User Documentation

- Update README.md
- Add usage examples
- Write FAQ documentation

## 🔒 Security

### Security Best Practices

- Sensitive data (like passwords) must be encrypted
- All network communication uses HTTPS
- Verify integrity and source of downloaded files
- Regularly update dependencies

### Security Vulnerability Reporting

If you discover a security vulnerability, please report it through secure channels:
- Email security@example.com
- Do not publicly disclose vulnerability details

## 🤝 Code of Conduct

We follow the Contributor Covenant Code of Conduct. Please ensure:

- Respect all community members
- Constructive discussion of technical issues
- Help new contributors integrate into the community

## 📞 Contact Information

- Issues: [GitHub Issues](https://github.com/pfinalcub/phpx/issues)
- Discussions: [GitHub Discussions](https://github.com/pfinalcub/phpx/discussions)
- Email: maintainers@example.com

---

Thank you for contributing! Let's make phpx better together! 🚀