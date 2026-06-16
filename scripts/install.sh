#!/usr/bin/env bash
# ── Git Hero Installer ───────────────────────────────────────────────
# One-liner install script for end users
# Usage: curl -fsSL https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.sh | bash
#
# Or locally: ./scripts/install.sh [version]

set -euo pipefail

VERSION="${1:-latest}"
BINARY="git-hero"
REPO="MarlonRX/git-hero"
INSTALL_DIR="${HOME}/.local/bin"

# ── Colors ────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}══════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Git Hero Installer${NC}"
echo -e "${CYAN}══════════════════════════════════════════════${NC}"
echo ""

# ── Detect platform ──────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)  PLATFORM="linux" ;;
    Darwin) PLATFORM="macos" ;;
    *)
        echo -e "${RED}❌ Unsupported OS: ${OS}${NC}"
        echo "   Git Hero currently supports Linux and macOS."
        exit 1
        ;;
esac

case "${ARCH}" in
    x86_64)     ARCH_NAME="x86_64" ;;
    arm64|aarch64) ARCH_NAME="arm64" ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: ${ARCH}${NC}"
        exit 1
        ;;
esac

# ── Determine version ────────────────────────────────────────────────
if [ "${VERSION}" = "latest" ]; then
    echo "🔍 Fetching latest version..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | \
        grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/')
    
    if [ -z "${VERSION}" ]; then
        echo -e "${RED}❌ Could not determine latest version.${NC}"
        echo "   Specify a version: ./install.sh 0.1.0"
        exit 1
    fi
fi

echo "📦 Version: v${VERSION}"
echo "💻 Platform: ${PLATFORM}-${ARCH_NAME}"
echo ""

# ── Create install directory ─────────────────────────────────────────
mkdir -p "${INSTALL_DIR}"

# ── Download ─────────────────────────────────────────────────────────
ARCHIVE_NAME="${BINARY}-v${VERSION}-${PLATFORM}-${ARCH_NAME}"
EXT="tar.gz"
[ "${PLATFORM}" = "linux" ] && EXT="tar.xz"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARCHIVE_NAME}.${EXT}"
CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

echo "⬇️  Downloading ${ARCHIVE_NAME}.${EXT}..."
TEMP_DIR="$(mktemp -d)"
curl -fsSL "${DOWNLOAD_URL}" -o "${TEMP_DIR}/${ARCHIVE_NAME}.${EXT}"

# ── Verify checksum (optional, non-fatal) ────────────────────────────
echo "🔐 Verifying checksum..."
if curl -fsSL "${CHECKSUM_URL}" -o "${TEMP_DIR}/checksum.txt" 2>/dev/null; then
    (cd "${TEMP_DIR}" && shasum -a 256 -c checksum.txt 2>/dev/null) || \
        echo -e "${RED}⚠️  Checksum verification failed. Proceeding anyway...${NC}"
else
    echo "⚠️  Could not download checksum. Skipping verification."
fi

# ── Extract ──────────────────────────────────────────────────────────
echo "📂 Extracting..."
if [ "${EXT}" = "tar.gz" ]; then
    tar -xzf "${TEMP_DIR}/${ARCHIVE_NAME}.${EXT}" -C "${TEMP_DIR}"
else
    tar -xJf "${TEMP_DIR}/${ARCHIVE_NAME}.${EXT}" -C "${TEMP_DIR}"
fi

# ── Install ──────────────────────────────────────────────────────────
echo "📥 Installing to ${INSTALL_DIR}/${BINARY}..."
cp "${TEMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
chmod +x "${INSTALL_DIR}/${BINARY}"

# ── Cleanup ──────────────────────────────────────────────────────────
rm -rf "${TEMP_DIR}"

# ── Verify installation ──────────────────────────────────────────────
if command -v "${INSTALL_DIR}/${BINARY}" >/dev/null 2>&1; then
    echo ""
    echo -e "${GREEN}══════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ✅ Git Hero v${VERSION} installed successfully!${NC}"
    echo -e "${GREEN}══════════════════════════════════════════════${NC}"
    echo ""
    echo "  Binary: ${INSTALL_DIR}/${BINARY}"
    echo ""
    
    # Check if install dir is in PATH
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        echo -e "${CYAN}💡 Add ${INSTALL_DIR} to your PATH:${NC}"
        echo "   echo 'export PATH=\"\${HOME}/.local/bin:\${PATH}\"' >> ~/.zshrc"
        echo "   source ~/.zshrc"
        echo ""
    fi
    
    echo "  Run: git-hero"
else
    echo -e "${RED}❌ Installation failed.${NC}"
    exit 1
fi
