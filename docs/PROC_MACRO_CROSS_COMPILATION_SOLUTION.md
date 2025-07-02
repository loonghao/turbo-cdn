# Proc-Macro 跨平台编译解决方案

## 问题描述

在跨平台编译 turbo-cdn 项目时遇到以下错误：

```
error: cannot produce proc-macro for `async-stream-impl v0.3.6` as the target `x86_64-unknown-linux-gnu` does not support these crate types
```

## 根本原因

1. **Proc-macros 必须在宿主平台编译**：proc-macro 是编译器插件，必须在编译机器上运行
2. **全局 CARGO_BUILD_TARGET 设置影响了 proc-macro**：当设置了全局目标时，Cargo 试图为目标平台编译 proc-macro
3. **async-stream-impl 是一个 proc-macro crate**：它通过 tokio-test 间接依赖进来

## 依赖链分析

```
tokio-test v0.4.4
├── async-stream v0.3.6
│   ├── async-stream-impl v0.3.6 (proc-macro)  ← 这里是问题所在
```

## 解决方案

### 1. Cross.toml 配置优化

我们的 `Cross.toml` 已经包含了正确的配置：

```toml
[build]
# Critical: Do NOT set default-target globally as it affects proc-macros

[build.env]
passthrough = [
    "CARGO_TARGET_DIR",
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
    "RUSTFLAGS",
    "OPENSSL_STATIC",
    "OPENSSL_NO_VENDOR",
    "PKG_CONFIG_ALLOW_CROSS",
    "CARGO_BUILD_TARGET",
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
]

# 每个目标平台的配置
[target.x86_64-unknown-linux-gnu]
pre-build = [
    "rustup target add x86_64-unknown-linux-gnu",
]

[target.x86_64-unknown-linux-gnu.env]
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = ""
```

### 2. CI/CD 配置

使用 rust-actions-toolkit v4.0.0 正式版，它包含了增强的 proc-macro 修复：

```yaml
jobs:
  # 原生测试（避免跨平台编译环境变量干扰）
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Fix proc-macro environment
        run: |
          chmod +x scripts/fix-proc-macro-env.sh
          ./scripts/fix-proc-macro-env.sh
      - name: Run tests
        run: cargo test --all-features --workspace

  # 跨平台编译测试
  cross-platform-test:
    needs: ci
    uses: loonghao/rust-actions-toolkit/.github/workflows/reusable-release.yml@v4.0.0
    with:
      binary-name: "turbo-cdn"
      enable-python-wheels: false
```

### 3. 环境修复脚本

我们创建了专门的脚本 `scripts/fix-proc-macro-env.sh` 来处理环境变量问题：

```bash
#!/bin/bash
# 自动检测和修复可能导致 proc-macro 问题的环境变量
./scripts/fix-proc-macro-env.sh
```

### 4. 本地测试

**环境修复测试：**
```bash
# 修复 proc-macro 环境
./scripts/fix-proc-macro-env.sh

# 验证修复效果
cargo test --all-features --workspace
```

**跨平台编译测试：**
```powershell
# Windows PowerShell
.\scripts\test-cross-compilation.ps1 -Target x86_64-unknown-linux-gnu -Verbose
```

```bash
# Linux/macOS
./scripts/test-cross-compilation.sh x86_64-unknown-linux-gnu
```

### 5. 手动修复步骤

如果自动修复不工作，可以手动执行：

```bash
# 1. 确保宿主工具链可用
rustup target add x86_64-unknown-linux-gnu

# 2. 设置环境变量
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=""

# 3. 使用 cross 编译
cross build --target x86_64-unknown-linux-gnu
```

## 验证修复

1. **本地编译测试**：`cargo build` ✅ (已验证成功)
2. **跨平台编译测试**：需要在 CI 环境中验证
3. **所有目标平台测试**：通过 CI 管道验证

## 相关文档

- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Cargo Build Configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- [Proc-Macro Book](https://doc.rust-lang.org/reference/procedural-macros.html)
- [rust-actions-toolkit Proc-Macro Fix](C:\Users\hallo\Documents\augment-projects\rust-release-action\docs\PROC_MACRO_CROSS_COMPILATION_FIX.md)

## 下一步

1. 提交当前的修复配置
2. 在 CI 中测试跨平台编译
3. 如果仍有问题，考虑升级到最新的 cross 版本或使用替代方案
