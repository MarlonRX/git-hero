#!/usr/bin/env bash
# ── Git Hero Release Builder ──────────────────────────────────────────
# Builds optimized release binaries for the current platform
# Usage: ./scripts/build-release.sh [version]
# Example: ./scripts/build-release.sh 0.1.0
#
# Prerequisites:
#   - Rust toolchain (rustup)

set -euo pipefail

# ── Colors & Styles ──────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
RED='\033[0;31m'
NC='\033[0m'

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"
BINARY="git-hero"
RELEASE_DIR="target/releases/${VERSION}"
DIST_DIR="target/dist/${VERSION}"

# ── Banner ───────────────────────────────────────────────────────────
echo ""
echo -e "${BLUE}${BOLD}"
echo "  ╔══════════════════════════════════════════════════════╗"
echo "  ║                                                      ║"
printf "  ║   Git Hero · Local Build v%-25s ║\n" "${VERSION}"
echo "  ║                                                      ║"
echo "  ╚══════════════════════════════════════════════════════╝"
echo -ne "${NC}"
echo ""

# ── Check prerequisites ─────────────────────────────────────────────
command -v cargo >/dev/null 2>&1 || {
    echo -e "  ${RED}✘  cargo not found. Install Rust: https://rustup.rs${NC}"
    exit 1
}

# ── Build release binary ────────────────────────────────────────────
echo -e "  ${WHITE}${BOLD}Building release binary (optimized)${NC}"
echo -e "  ${DIM}$(printf '%.0s─' {1..50})${NC}"
cargo build --release
echo -e "  ${GREEN}✔${NC}  Release binary: ${DIM}target/release/${BINARY}${NC}"
echo ""

# ── Create distribution directory ───────────────────────────────────
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# ── Platform detection ──────────────────────────────────────────────
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

# ── Package binary ──────────────────────────────────────────────────
ARCHIVE_NAME="${BINARY}-v${VERSION}-${PLATFORM}-${ARCH_NAME}"

if [ "${PLATFORM}" = "macos" ]; then
    TARBALL="${ARCHIVE_NAME}.tar.gz"
else
    TARBALL="${ARCHIVE_NAME}.tar.xz"
fi

echo -e "  ${WHITE}${BOLD}Packaging${NC}"
echo -e "  ${DIM}$(printf '%.0s─' {1..50})${NC}"
echo -e "  ${DIM}Archive:${NC} ${TARBALL}"

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

# ── Generate checksums ──────────────────────────────────────────────
echo -e "  ${DIM}Checksum:${NC}"
shasum -a 256 "${DIST_DIR}/${TARBALL}" > "${DIST_DIR}/${TARBALL}.sha256"
echo -e "  ${DIM}$(cat "${DIST_DIR}/${TARBALL}.sha256")${NC}"

# ── Summary ─────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}"
echo "  ╔══════════════════════════════════════════════════════╗"
echo "  ║                                                      ║"
printf "  ║   ✔  Release v%-37s ║\n" "${VERSION} built"
echo "  ║                                                      ║"
echo "  ╚══════════════════════════════════════════════════════╝"
echo -ne "${NC}"
echo ""
echo -e "  ${DIM}Binary${NC}     target/release/${BINARY}"
echo -e "  ${DIM}Package${NC}    ${DIST_DIR}/${TARBALL}"
echo -e "  ${DIM}Checksum${NC}   ${DIST_DIR}/${TARBALL}.sha256"
echo -e "  ${DIM}Size${NC}       $(du -h "target/release/${BINARY}" | cut -f1)"
echo ""
echo -e "  ${WHITE}${BOLD}To publish on GitHub:${NC}"
echo -e "  ${DIM}1.${NC} ${CYAN}git tag -a v${VERSION} -m \"Release v${VERSION}\"${NC}"
echo -e "  ${DIM}2.${NC} ${CYAN}git push origin v${VERSION}${NC}"
echo -e "  ${DIM}3.${NC} Upload ${TARBALL} to the GitHub release page"
echo -e "  ${DIM}4.${NC} Upload ${TARBALL}.sha256 as well"
echo ""
echo -e "  ${WHITE}${BOLD}To publish on crates.io:${NC}"
echo -e "  ${DIM}\$${NC} ${CYAN}cargo publish${NC}"
echo ""
