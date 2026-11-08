#!/usr/bin/env bash

set -e

usage() {
    echo "usage: $0 [debug|release]"
    echo ""
    echo "  debug    - build in debug mode (default)"
    echo "  release  - build in release mode"
    exit 1
}

MODE="${1:-debug}"

case "$MODE" in
    debug|release) ;;
    -h|--help) usage ;;
    *) echo "error: unknown mode '$MODE'" >&2; usage ;;
esac

if ! command -v tar >/dev/null 2>&1; then
    echo "error: tar is not installed or not available in PATH" >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo is not installed or not available in PATH" >&2
    exit 1
fi

if ! command -v xorriso >/dev/null 2>&1; then
    echo "error: xorriso is not installed or not available in PATH" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

if [ ! -f iso/boot/limine-uefi-cd.bin ]; then
    echo "limine-uefi-cd.bin not found, running install.sh..."
    bash "$SCRIPT_DIR/install.sh"
fi

tar -cf iso/boot/initramfs.tar initramfs/*

if [ "$MODE" = "release" ]; then
    cargo build --release
    cp target/x86_64-unknown-none/release/regesa \
       iso/boot/regesa.elf
else
    cargo build
    cp target/x86_64-unknown-none/debug/regesa \
       iso/boot/regesa.elf
fi

xorriso -as mkisofs \
  --efi-boot boot/limine-uefi-cd.bin \
  -efi-boot-part \
  --efi-boot-image \
  --protective-msdos-label \
  iso \
  -o regesa.iso

echo "build complete: regesa.iso ($MODE)"
