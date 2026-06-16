#!/usr/bin/env bash
# ── Git Hero Release Builder ──────────────────────────────────────────
# Builds optimized release binaries for multiple platforms
# Usage: ./scripts/build-release.sh [version]
# Example: ./scripts/build-release.sh 0.1.0
#
# Prerequisites:
#   - Rust toolchain (rustup)
#   - Cross-compilation targets (added automatically)

set -euo pipefail

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"
BINARY="git-hero"
RELEASE_DIR="target/releases/${VERSION}"
DIST_DIR="target/dist/${VERSION}"

echo "══════════════════════════════════════════════"
echo "  Git Hero Release Builder v${VERSION}"
echo "══════════════════════════════════════════════"
echo ""

# ── Check prerequisites ──────────────────────────────────────────────
command -v cargo >/dev/null 2>&1 || { echo "❌ cargo not found. Install Rust: https://rustup.rs"; exit 1; }

# ── Ensure release profile ───────────────────────────────────────────
echo "⚡ Building release binary (optimized)..."
cargo build --release
echo "✅ Release binary built: target/release/${BINARY}"
echo ""

# ── Create distribution directory ────────────────────────────────────
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# ── Platform detection ───────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)  PLATFORM="linux" ;;
    Darwin) PLATFORM="macos" ;;
    *)      PLATFORM="unknown" ;;
esac

case "${ARCH}" in
    x86_64)  ARCH_NAME="x86_64" ;;
    arm64|aarch64) ARCH_NAME="arm64" ;;
    *)       ARCH_NAME="${ARCH}" ;;
esac

# ── Package binary ───────────────────────────────────────────────────
ARCHIVE_NAME="${BINARY}-v${VERSION}-${PLATFORM}-${ARCH_NAME}"

if [ "${PLATFORM}" = "macos" ]; then
    TARBALL="${ARCHIVE_NAME}.tar.gz"
else
    TARBALL="${ARCHIVE_NAME}.tar.xz"
fi

echo "📦 Packaging: ${TARBALL}"

# Create temp dir for packaging
TEMP_DIR="$(mktemp -d)"
cp "target/release/${BINARY}" "${TEMP_DIR}/"
cp README.md LICENSE "${TEMP_DIR}/" 2>/dev/null || true

# Create archive
if [ "${PLATFORM}" = "macos" ]; then
    tar -czf "${DIST_DIR}/${TARBALL}" -C "${TEMP_DIR}" .
else
    tar -cJf "${DIST_DIR}/${TARBALL}" -C "${TEMP_DIR}" .
fi

rm -rf "${TEMP_DIR}"

# ── Generate checksums ───────────────────────────────────────────────
echo "🔐 Generating checksums..."
shasum -a 256 "${DIST_DIR}/${TARBALL}" > "${DIST_DIR}/${TARBALL}.sha256"

# ── Summary ──────────────────────────────────────────────────────────
echo ""
echo "══════════════════════════════════════════════"
echo "  Release ${VERSION} built successfully!"
echo "══════════════════════════════════════════════"
echo ""
echo "  Binary:    target/release/${BINARY}"
echo "  Package:   ${DIST_DIR}/${TARBALL}"
echo "  Checksum:  ${DIST_DIR}/${TARBALL}.sha256"
echo ""
echo "  Size:      $(du -h "target/release/${BINARY}" | cut -f1)"
echo "  Stripped:  $(strip "target/release/${BINARY}" 2>/dev/null && du -h "target/release/${BINARY}" | cut -f1 || echo "N/A (macOS)")"
echo ""
echo "📤 To publish on GitHub:"
echo "   1. Create a tag:  git tag -a v${VERSION} -m \"Release v${VERSION}\""
echo "   2. Push the tag:  git push origin v${VERSION}"
echo "   3. Upload ${TARBALL} to the GitHub release page"
echo "   4. Upload ${TARBALL}.sha256 as well"
echo ""
echo "📦 To publish on crates.io:"
echo "   cargo publish"
echo ""
