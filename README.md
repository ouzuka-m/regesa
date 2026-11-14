# Regesa

**64-bit Operating System Kernel in Rust**

[![Rust Kernel CI](https://github.com/ouzuka-m/regesa/actions/workflows/rust.yml/badge.svg)](https://github.com/ouzuka-m/regesa/actions/workflows/rust.yml)
![License](https://img.shields.io/badge/license-GPL--2.0-blue)
![Architecture](https://img.shields.io/badge/arch-x86__64-orange)
![Rust](https://img.shields.io/badge/rust-nightly-red)
![Boot](https://img.shields.io/badge/boot-UEFI%20%2B%20Limine-green)

## What is Regesa?

Regesa is an operating system kernel written in Rust, targeting x86_64 systems with UEFI boot via the [Limine](https://github.com/limine-bootloader/limine) boot protocol. The project focuses on safety and modern kernel design.

> [!NOTE]
> Regesa is in early, active development. Expect frequent breaking changes.

## Hardware Requirements

- 64-bit Intel CPU with **TSC-Deadline** timer support
- 2GB or more RAM
- **UEFI** firmware (legacy BIOS is **not** supported)
- **KVM** for virtualized testing (QEMU TCG is insufficient — TSC-Deadline is not fully emulated)

## Prerequisites

| Tool                     | Purpose                                |
| ------------------------ | -------------------------------------- |
| `rustc` + `cargo`        | Rust compiler and build system (nightly, pinned via `rust-toolchain.toml`) |
| `xorriso`                | Creating bootable ISO images           |
| `tar`                    | Packing the initial root filesystem    |
| `qemu-system-x86_64`    | Running the kernel in a virtual machine |
| OVMF                     | UEFI firmware for QEMU                 |

### Host Platform

> [!IMPORTANT]
> **Linux** is the only supported build and development platform.
> Windows and macOS have not been tested and are not supported.

### Installing Prerequisites (Arch Linux)

```bash
sudo pacman -S qemu-full xorriso edk2-ovmf
```

### Installing Prerequisites (Debian / Ubuntu)

```bash
sudo apt install qemu-system-x86 xorriso ovmf
```

Rust nightly is managed automatically via [`rust-toolchain.toml`](rust-toolchain.toml) — simply install [rustup](https://rustup.rs/) and the correct toolchain will be used.

## Building

Build scripts are located in the [`scripts/`](scripts/) directory.

### Debug Build

```bash
./scripts/build.sh debug
```

Compiles the kernel in debug mode and creates `regesa.iso`.

### Release Build

```bash
./scripts/build.sh release
```

Compiles the kernel with optimizations and creates `regesa.iso`.

Both scripts will:
1. Pack `rootfs/` into `iso/boot/initramfs.tar`
2. Compile the kernel ELF binary
3. Generate a bootable ISO with Limine

## Running with QEMU

After building, launch the ISO with QEMU + OVMF:

```bash
qemu-system-x86_64 \
  -bios /usr/share/edk2/x64/OVMF.fd \
  -cdrom regesa.iso \
  -cpu host \
  -m 2G \
  -enable-kvm
```

To capture serial output in the terminal:

```bash
qemu-system-x86_64 \
  -bios /usr/share/edk2/x64/OVMF.fd \
  -cdrom regesa.iso \
  -cpu host \
  -enable-kvm \
  -m 2G \
  -serial stdio
```

> [!TIP]
> The OVMF firmware path varies by distro. Common paths:
> - Arch Linux — `/usr/share/edk2/x64/OVMF.fd`
> - Ubuntu/Debian — `/usr/share/OVMF/OVMF_CODE.fd`

## Kernel Boot Sequence

1. **Bootloader Hand-off (Limine)**: The firmware loads Limine, which fulfills the static requests in [`src/boot/requests.rs`](src/boot/requests.rs) (HHDM offset, memory map, RSDP, framebuffer, and boot modules), switches the CPU to 64-bit long mode, and jumps to `_start`.
2. **Early CPU Architecture**: Disables CPU interrupts (`cli`), initializes COM1 UART serial output for logging, configures the Global Descriptor Table (GDT) and Task State Segment (TSS) with dedicated Interrupt Stack Table (IST) stacks, and loads the Interrupt Descriptor Table (IDT).
3. **Memory Management**: Constructs the virtual memory page mapper via HHDM from `CR3`, builds a bitmap-based physical frame allocator from the memory map, maps 1 MiB of pages starting at `0xFFFF_9000_0000_0000`, and initializes the buddy heap allocator.
4. **Drivers & Interrupt Routing**: Parses ACPI tables from RSDP, disables legacy 8259 PICs, initializes the High Precision Event Timer (HPET), enables the Local APIC, calibrates and arms the TSC-Deadline timer, and routes PS/2 keyboard IRQs through the I/O APIC.
5. **Jump to Init Program**: Parses `initramfs.tar` (USTAR module) via TarFS, displays system hardware information and CPU brand on the framebuffer, triggers QEMU debug-exit (for automated CI testing), executes init program (`/bin/init.bin`), prints the prompt (`$ `), activates scheduler, and enters idle loop (`lt`).

## Directory Layout

```
regesa/
├── iso/            ISO structure with Limine bootloader binaries
├── initramfs/      Root filesystem files packed into initramfs
├── scripts/        Build & environment setup scripts (build.sh)
├── src/              Kernel source code
│   ├── boot/         Boot initialization
│   ├── config/       Kernel configuration & memory metadata
│   ├── display/      Framebuffer and text rendering
│   ├── drivers/      Device drivers
│   ├── fs/           Filesystems
│   ├── log/          Kernel logging
│   ├── memory/       Frame allocator, mapper, heap
│   ├── process/      ELF loader & process spawning
│   ├── qemu/         QEMU integration utilities
│   ├── scheduler/    Scheduler and task management
│   ├── x86_64/       x86_64 arch-specific code
│   ├── error.rs       Unified kernel error types
│   └── main.rs       Kernel entry point
├── targets/        Custom target specification (x86_64-unknown-none.json)
├── linker.ld       Linker script for kernel section layout
├── Cargo.toml      Package manifest and dependencies
├── README.md
└── rust-toolchain.toml  Pinned nightly Rust toolchain
```

## Contributing — We Need You!

Regesa is an ambitious, experimental kernel under active development. Building an operating system from the ground up is a huge challenge, and **we need your help to make it rock solid**. Whether you are an experienced systems hacker or exploring OS development in Rust for the first time, there are plenty of high-impact areas where you can jump in right away:

- **🧪 Testing & Hardware Verification**:
  Regesa does not have an automated test suite yet. You can help by running manual boot tests across diverse CPU and QEMU/KVM setups, testing both debug and release builds, monitoring serial output, and hunting down panics, freezes, or regressions.
- **🛡️ Auditing & Safe Abstractions (`unsafe`)**:
  Low-level hardware access inevitably requires `unsafe`, but we strive to keep it minimal. Help us audit raw pointers and MMIO/register accesses, wrap low-level routines into sound, idiomatic safe abstractions, and document safety invariants with `// SAFETY:` comments.
- **📦 Dependency & Toolchain Maintenance**:
  The Rust nightly ecosystem moves fast. We need contributors to track outdated dependencies (`cargo outdated`), test nightly compiler compatibility, and keep toolchain pins synchronized with CI.
- **✨ Code Quality & Linting**:
  We enforce a strict **zero-warnings, zero-errors** policy (`cargo clippy -- -D warnings`). Help polish existing code, adopt more idiomatic patterns, improve formatting (`cargo fmt`), and expand inline documentation.
- **⚙️ Kernel Subsystem & Driver Expansion**:
  Help advance Regesa's core capabilities — including physical/virtual memory management, scheduling, filesystem features (beyond initramfs), interrupt controllers (APIC/HPET), and hardware drivers.

Ready to contribute? Check out [CONTRIBUTING.md](CONTRIBUTING.md) for our detailed guidelines, code standards, and PR workflow.

## License

Regesa is licensed under the [GNU General Public License v2.0 (GPL-2.0)](LICENSE).
