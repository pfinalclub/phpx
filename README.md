# phpx

> **The missing tool runner for PHP.**
> Run PHP CLI tools without installing them. Version-locked. Reproducible. CI-safe.

---

## Why phpx exists (the real problem)

If you work with PHP long enough, you already know this pain:

* `composer global` pollutes machines and breaks across teams
* `vendor/bin` ties tools to project deps (and explodes upgrade cost)
* CI and local environments silently run **different tool versions**
* Debugging tool-version drift wastes hours and creates false failures

**phpx fixes one thing, brutally well:**

> It guarantees that PHP CLI tools run with the exact same version — locally, in CI, everywhere.

No global installs. No Composer coupling. No excuses.

---

## What phpx is (and is not)

### phpx **is**

* A zero-install **PHP tool runner** (like `npx`, but for PHP)
* A single static binary (fast, predictable, portable)
* A version-locked execution layer for tools like:

  * phpstan
  * php-cs-fixer
  * psalm
  * rector
  * pest

### phpx **is not**

* ❌ A framework
* ❌ A package manager replacement
* ❌ Another abstraction over Composer

It does one job: **run PHP tools correctly**.

---

## The 10-second example

```bash
# Run phpstan without installing anything
phpx phpstan@1.11 analyse src
```

That’s it.

* phpstan is downloaded if missing
* cached locally
* executed with the exact version you requested

---

## Why not just use Composer?

Because Composer solves a *different* problem.

| Problem                | Composer   | phpx      |
| ---------------------- | ---------- | --------- |
| Project dependencies   | ✅          | ❌         |
| Tool version isolation | ⚠️ painful | ✅ trivial |
| Global installs        | ❌ fragile  | ✅ avoided |
| CI reproducibility     | ⚠️ manual  | ✅ default |
| Temporary tool runs    | ❌          | ✅         |

**Rule of thumb**:

> If the tool is not part of your runtime, it should not live in your dependency graph.

---

## CI-first by design

phpx was designed assuming CI will break first.

Example (GitHub Actions):

```yaml
- name: Run PHPStan
  run: |
    phpx phpstan@1.11 analyse src
```

No setup steps. No Composer hacks. No version drift.

---

## Deterministic builds (coming next)

phpx is moving toward **lockfile-based toolchains**.


```bash
phpx phpstan
```

This turns phpx from a runner into **infrastructure**.

---

## Installation

```bash
curl -fsSL https://github.com/pfinalclub/phpx/releases/latest/download/phpx \
  -o /usr/local/bin/phpx && chmod +x /usr/local/bin/phpx
```

(Windows and macOS binaries are provided in releases.)

---

## Supported sources

phpx can fetch tools from:

* Packagist (phar packages)
* GitHub Releases
* Direct URLs

All downloads are cached and checksum-verified.

---

## When you should use phpx

Use phpx if:

* You maintain PHP projects across multiple machines
* Your CI breaks because of tool version mismatch
* You are tired of `composer global`
* You build automation, agents, or ephemeral environments

Don’t use phpx if:

* You want another framework
* You expect a GUI
* You enjoy debugging environment issues

---

## Design philosophy

* **One binary**
* **No background services**
* **No magic state**
* **Fail loudly**

phpx prefers boring, predictable behavior over clever abstractions.

---

## Status

phpx is actively developed and used in real projects.

It is intentionally small.
Its surface area is kept minimal.

If you want features, bring real-world use cases.

---

## Contributing

Issues and PRs are welcome — especially:

* CI integrations
* Tool-specific docs (phpstan, psalm, rector, etc.)
* Lockfile design feedback

---

## License

MIT
