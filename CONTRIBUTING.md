# Contributing to Regesa

Thank you for your interest in contributing to Regesa! This document outlines the guidelines and requirements that **every contributor must follow** to maintain codebase quality and stability.

---

## Table of Contents

- [Codebase Stabilization](#codebase-stabilization)
- [Toolchain Management](#toolchain-management)
- [Dependency Tracking](#dependency-tracking)
- [Code Quality Standards](#code-quality-standards)
- [Usage of `unsafe`](#usage-of-unsafe)
- [Manual Verification](#manual-verification)
- [Contribution Workflow](#contribution-workflow)

---

## Codebase Stabilization

Regesa is an experimental kernel under active development. Keeping the codebase stable is a top priority.

Core principles:

- **Zero warnings, zero errors** — every push and merge **must not** produce any compiler warnings or errors.
- **Do not blindly add features** — ensure that new functionality does not break what already exists. Since the kernel has no test suite, manual verification is the contributor's responsibility.
- **Regressions are treated seriously** — if a change causes the kernel to fail to boot, hang, panic, or exhibit any other unexpected behavior, the change will be rejected.

---

## Toolchain Management

Regesa uses **Rust nightly** because it depends on unstable features (such as `abi_x86_interrupt`). The toolchain is pinned to a specific nightly date to ensure reproducible builds.

### Source of Truth

The toolchain version is defined in two places, and **both must always be in sync**:

| File | Role |
| --- | --- |
| [`rust-toolchain.toml`](rust-toolchain.toml) | Local developer toolchain (used by `rustup`) |
| [`.github/workflows/rust.yml`](.github/workflows/rust.yml) | CI toolchain (GitHub Actions) |

Example of a correct, synchronized state:

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly-2026-10-01"
components = ["rustfmt", "clippy", "rust-src"]
```

```yaml
# .github/workflows/rust.yml
- uses: dtolnay/rust-toolchain@master
  with:
    toolchain: nightly-2026-09-01
    components: rustfmt, clippy, rust-src
```

> [!CAUTION]
> **Never** change the nightly version in one file without updating the other. A mismatch between `rust-toolchain.toml` and `rust.yml` leads to unreliable CI — code may pass locally but fail in CI, or vice versa.

### Toolchain Update Procedure

When bumping the nightly version:

1. Update `channel` in `rust-toolchain.toml`
2. Update `toolchain` in `.github/workflows/rust.yml`
3. Ensure `components` are identical in both files
4. Run a full local build (`./build/debug.sh` and `./build/release.sh`)
5. Run `cargo clippy -- -D warnings` and `cargo fmt --check`
6. Commit both files in a **single commit**

---

## Dependency Tracking

Crate dependencies must be tracked and updated regularly to stay current, especially since the nightly Rust ecosystem can introduce breaking changes at any time.

### Guidelines

- **Check for outdated crates periodically** — use `cargo outdated` (install via `cargo install cargo-outdated`) to identify dependencies that are behind.
- **Watch for nightly compatibility** — some crates may break on specific nightly versions. Always test after updating.
- **Bump one crate per commit** — this makes it easier to bisect if a regression occurs.
- **Review changelogs and breaking changes** — before bumping a major version, read the crate's changelog.

### Current Crate Overview

| Crate | Purpose |
| --- | --- |
| `limine` | Limine boot protocol bindings |
| `x86_64` | x86_64 architecture abstractions |
| `spin` | Spinlock primitives (Mutex, Once, LazyLock) |
| `uart_16550` | UART serial port driver |
| `pc-keyboard` | PS/2 keyboard driver |
| `noto-sans-mono-bitmap` | Bitmap font for framebuffer rendering |
| `buddy_system_allocator` | Buddy system heap allocator |
| `acpi` | ACPI table parsing |

---

## Code Quality Standards

Every pull request and commit **must** meet the following standards:

### 1. No Warnings and No Errors

Code must compile without any warnings or errors. CI enforces this with:

```bash
cargo clippy -- -D warnings
```

The `-D warnings` flag promotes all warnings to errors — **a single warning is enough to fail CI**.

### 2. Code Formatting with `cargo fmt`

All code must be formatted using `rustfmt`. CI checks this with:

```bash
cargo fmt --check
```

Before committing, **always run**:

```bash
cargo fmt
```

> [!TIP]
> Set up `rustfmt` in your editor or IDE to format automatically on save.

### 3. Linting with `cargo clippy`

Clippy is the official Rust linter. Beyond catching warnings, it also suggests more idiomatic and safer code patterns.

```bash
cargo clippy -- -D warnings
```

### Pre-Push Checklist

```bash
cargo fmt                        # Format code
cargo clippy -- -D warnings      # Lint and check for warnings
./scripts/build.sh debug         # Debug build
./scripts/build.sh release       # Release build
```

All four steps above **must** succeed without errors before pushing.

---

## Usage of `unsafe`

In kernel development, `unsafe` is sometimes unavoidable — register access, MMIO, inline assembly, and similar operations require it. However:

- **Minimize `unsafe` wherever possible** — if there is a safe way to achieve the same result, use the safe approach.
- **Wrap `unsafe` in safe abstractions** — do not scatter `unsafe` blocks throughout the codebase. Create safe wrapper functions that encapsulate unsafe operations.
- **Document every `unsafe` block** — write a `// SAFETY: ...` comment explaining why the operation is sound despite the compiler being unable to verify it.

Good example:

```rust
/// Reads the value of the CR3 register.
pub fn read_cr3() -> PhysFrame {
    // SAFETY: Reading CR3 is a non-destructive operation that does
    // not modify any state. It simply returns the current page table
    // base address.
    unsafe { x86_64::registers::control::Cr3::read().0 }
}
```

---

## Manual Verification

> [!WARNING]
> Regesa **does not have a test suite or testing framework**. This means every change must be manually verified by the contributor.

Since there is no automated testing, contributors are responsible for:

1. **Boot test** — ensure the kernel still boots after your changes.
   ```bash
   ./scripts/build.sh
   qemu-system-x86_64 \
     -bios /usr/share/edk2/x64/OVMF.fd \
     -cdrom regesa.iso \
     -cpu host \
     -enable-kvm \
     -m 2G \
     -serial stdio
   ```

2. **Check serial output** — review the serial log for panics, errors, or unexpected behavior.

3. **Observe runtime behavior** — verify that:
   - The kernel does not hang or freeze
   - There are no kernel panics
   - Display output (framebuffer) works correctly
   - Interrupt handling functions as expected (keyboard, timer)

4. **Test with the release build as well** — certain bugs only appear in release builds due to compiler optimizations.
   ```bash
   ./scripts/build.sh release
   ```

---

## Contribution Workflow

1. **Fork** the repository
2. Create a new **branch** from `main`:
   ```bash
   git checkout -b feat/feature-name
   ```
3. Make your changes following all the standards above
4. Ensure the entire quality checklist passes
5. **Commit** with a clear message following [Conventional Commits](https://www.conventionalcommits.org/):
   ```
   feat(subsystem): short description
   fix(subsystem): short description
   refactor(subsystem): short description
   chore(deps): short description
   docs(readme): short description
   ```
6. **Push** and open a **Pull Request**
7. Wait for review — CI must be green before merge

---

> Thank you for helping build Regesa! 🚀
