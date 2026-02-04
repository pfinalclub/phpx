# 贡献指南

感谢您对 phpx 项目的关注！我们欢迎各种形式的贡献。

## 🚀 快速开始

### 开发环境设置

1. **克隆项目**
   ```bash
   git clone https://github.com/pfinalcub/phpx.git
   cd phpx
   ```

2. **安装 Rust 工具链**
   - 安装 [Rust](https://www.rust-lang.org/tools/install) 1.70+
   - 安装必要的工具：
     ```bash
     rustup component add clippy
     rustup component add rustfmt
     ```

3. **构建项目**
   ```bash
   cargo build
   ```

## 🎯 开发流程

### 代码风格

- 遵循 Rust 官方代码风格
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 进行代码检查
- 函数命名采用 snake_case
- 每行代码不超过 80 字符

### 提交信息规范

使用约定式提交格式：

```
<类型>[可选的作用域]: <描述>

[可选的正文]

[可选的脚注]
```

**类型**:
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具变动

### 分支策略

- `main`: 主分支，稳定版本
- `develop`: 开发分支
- `feature/*`: 功能分支
- `fix/*`: bug 修复分支

## 🐛 报告问题

在报告问题前，请：

1. 检查是否已有相关 issue
2. 提供详细的复现步骤
3. 包含错误日志和系统信息
4. 如果可能，提供最小复现代码

## 🔧 开发任务

### 当前开发重点

#### Phase 2: 安全验证和配置系统完善
- [ ] 实现 GPG/PGP 签名验证
- [ ] 完善配置文件读写功能
- [ ] 实现 `phpx config` 子命令
- [ ] 实现 `phpx self-update` 命令

#### Phase 3: 高级功能和用户体验优化
- [ ] 添加进度条显示
- [ ] 实现缓存 TTL 和空间限制
- [ ] 支持 HTTP 代理
- [ ] 添加自动完成支持

### 如何开始贡献

1. **选择任务**: 从上面的任务列表中选择一个
2. **创建分支**: `git checkout -b feature/your-feature`
3. **实现功能**: 编写代码和测试
4. **运行测试**: `cargo test`
5. **代码检查**: `cargo clippy && cargo fmt`
6. **提交 PR**: 描述功能变更和测试情况

## 🧪 测试

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行集成测试
cargo test --test integration
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率测试
cargo tarpaulin --ignore-tests
```

## 📚 文档

### 代码文档

- 所有公共函数和复杂逻辑都应添加注释
- 使用 Rustdoc 格式编写文档注释
- 运行 `cargo doc` 生成文档

### 用户文档

- 更新 README.md
- 添加使用示例
- 编写常见问题解答

## 🔒 安全

### 安全最佳实践

- 敏感数据（如密码）需加密存储
- 所有网络通信使用 HTTPS
- 验证下载文件的完整性和来源
- 定期更新依赖项

### 安全漏洞报告

如果您发现安全漏洞，请通过安全渠道报告：
- 发送邮件至 security@example.com
- 不要公开披露漏洞细节

## 🤝 行为准则

我们遵循贡献者公约行为准则。请确保：

- 尊重所有社区成员
- 建设性讨论技术问题
- 帮助新贡献者融入社区

## 📞 联系方式

- Issues: [GitHub Issues](https://github.com/your-username/phpx/issues)
- 讨论: [GitHub Discussions](https://github.com/your-username/phpx/discussions)
- 邮件: maintainers@example.com

---

感谢您的贡献！让我们一起让 phpx 变得更好！ 🚀