# phpx

> **PHP 缺的那块拼图：工具运行器。**
> 不安装也能跑 PHP 命令行工具。版本可锁、可复现、CI 友好。

---

## 为什么要有 phpx（真正的问题）

用 PHP 久了，这些痛你一定懂：

* `composer global` 污染环境，团队间难以一致
* `vendor/bin` 把工具绑进项目依赖，升级成本爆炸
* CI 和本机悄悄跑着**不同版本**的工具
* 排查工具版本漂移耗时间，还带来误报

**phpx 只做一件事，并且做狠：**

> 保证 PHP 命令行工具在本地、CI、任何地方都用**同一版本**运行。

不全局安装。不绑 Composer。没有借口。

---

## phpx 是什么（以及不是什么）

### phpx **是**

* 零安装的 **PHP 工具运行器**（类似 `npx`，但面向 PHP）
* 单一静态二进制（快、可预期、可携带）
* 为以下工具提供版本锁定的执行层：

  * phpstan
  * php-cs-fixer
  * psalm
  * rector
  * pest

### phpx **不是**

* ❌ 框架
* ❌ 包管理器替代品
* ❌ 在 Composer 上再包一层

它只干一件事：**把 PHP 工具跑对**。

---

## 十秒示例

```bash
# 不安装任何东西，直接跑 phpstan
phpx phpstan@1.11 analyse src
```

就这么多。

* 没有就自动下载
* 本地缓存
* 按你指定的版本执行

---

## 为什么不用 Composer 就行？

因为 Composer 解决的是**另一类**问题。

| 问题               | Composer   | phpx      |
| ------------------ | ---------- | --------- |
| 项目依赖           | ✅          | ❌         |
| 工具版本隔离       | ⚠️ 很折腾  | ✅ 很简单  |
| 全局安装           | ❌ 易碎     | ✅ 不需要  |
| CI 可复现          | ⚠️ 手动搞  | ✅ 开箱即用 |
| 临时跑一下工具     | ❌          | ✅         |

**一句话：**

> 工具若不是运行时的一部分，就不该进依赖图。

---

## 为 CI 而生

phpx 的设计前提是：CI 会先崩。

示例（GitHub Actions）：

```yaml
- name: Run PHPStan
  run: |
    phpx phpstan@1.11 analyse src
```

不用装环境。不用 Composer 骚操作。没有版本漂移。

---

## 确定性构建（规划中）

phpx 正在向 **基于 lockfile 的工具链** 演进。

```bash
phpx phpstan
```

届时 phpx 会从「运行器」变成**基础设施**。

---

## 安装

```bash
curl -fsSL https://github.com/pfinalclub/phpx/releases/latest/download/phpx \
  -o /usr/local/bin/phpx && chmod +x /usr/local/bin/phpx
```

（Windows 与 macOS 二进制见 Releases。）

---

## 支持的来源

phpx 可从以下来源拉取工具：

* Packagist（phar 包）
* GitHub Releases
* 直接 URL

所有下载都会缓存并做校验和验证。

---

## 什么时候该用 phpx

适合用 phpx 的情况：

* 你在多台机器上维护 PHP 项目
* CI 老因工具版本不一致挂掉
* 你受够了 `composer global`
* 你在做自动化、Agent 或临时环境

不适合用 phpx 的情况：

* 你要的是又一个框架
* 你需要 GUI
* 你喜欢排查环境问题

---

## 设计原则

* **一个二进制**
* **无后台服务**
* **无隐藏状态**
* **出错就大声报**

phpx 宁可无聊、可预期，也不要聪明抽象。

---

## 状态

phpx 在持续开发，并在真实项目中使用。

刻意保持小巧，表面积极简。

想要新功能，请带上真实场景。

---

## 贡献

欢迎提 Issue 和 PR，尤其欢迎：

* CI 集成方案
* 单工具文档（phpstan、psalm、rector 等）
* Lockfile 设计反馈

详细说明见 [CONTRIBUTING_CN.md](CONTRIBUTING_CN.md)。

---

## 许可证

MIT
