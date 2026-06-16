#!/usr/bin/env bash
# ── Git Hero Installer ───────────────────────────────────────────────
# One-liner install script for end users
# Usage: curl -fsSL https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.sh | bash
#
# Or locally: ./scripts/install.sh [version]
#
# Strategy:
#   1. Try to install from prebuilt release binary (fast)
#   2. Fall back to `cargo install` (slower but always works)

set -u  # NOT -e: we handle errors explicitly

VERSION="${1:-latest}"
BINARY="git-hero"
REPO="MarlonRX/git-hero"
INSTALL_DIR="${HOME}/.local/bin"

# ── Colors ────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

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
    x86_64)       ARCH_NAME="x86_64" ;;
    arm64|aarch64) ARCH_NAME="arm64" ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: ${ARCH}${NC}"
        exit 1
        ;;
esac

echo "💻 Platform: ${PLATFORM}-${ARCH_NAME}"
echo ""

# ── Helper functions ─────────────────────────────────────────────────
have_cmd() { command -v "$1" >/dev/null 2>&1; }

curl_get() {
    # curl_get <url> <output_file>
    # Returns 0 on success, 1 on failure (no -f flag, no exit on error)
    curl -fsSL --max-time 30 "$1" -o "$2" 2>/dev/null
}

# ── Try prebuilt release ─────────────────────────────────────────────
install_from_release() {
    local version="$1"

    # If version is "latest", try to fetch it from the API
    if [ "${version}" = "latest" ]; then
        echo "🔍 Fetching latest version from GitHub..."
        local api_resp
        api_resp=$(curl -fsSL --max-time 15 "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null) || true

        if [ -n "${api_resp}" ]; then
            version=$(echo "${api_resp}" | grep '"tag_name":' | head -1 | sed -E 's/.*"v?([^"]+)".*/\1/')
        fi

        if [ -z "${version}" ] || [ "${version}" = "latest" ]; then
            echo -e "${YELLOW}⚠️  Could not determine latest version (GitHub API rate limit or no releases).${NC}"
            echo -e "   Falling back to: ${GREEN}cargo install${NC}"
            return 1
        fi
    fi

    echo "📦 Version: v${version}"

    local archive_name="${BINARY}-v${version}-${PLATFORM}-${ARCH_NAME}"
    local ext="tar.gz"
    [ "${PLATFORM}" = "linux" ] && ext="tar.xz"

    local download_url="https://github.com/${REPO}/releases/download/v${version}/${archive_name}.${ext}"
    local temp_dir
    temp_dir=$(mktemp -d)

    echo "⬇️  Downloading ${archive_name}.${ext}..."
    if ! curl_get "${download_url}" "${temp_dir}/${archive_name}.${ext}"; then
        echo -e "${YELLOW}⚠️  Release binary not found (${version} for ${PLATFORM}-${ARCH_NAME}).${NC}"
        echo -e "   Falling back to: ${GREEN}cargo install${NC}"
        rm -rf "${temp_dir}"
        return 1
    fi

    # Optional checksum verification
    local checksum_url="${download_url}.sha256"
    if curl_get "${checksum_url}" "${temp_dir}/checksum.txt"; then
        if (cd "${temp_dir}" && shasum -a 256 -c checksum.txt >/dev/null 2>&1); then
            echo "🔐 Checksum OK"
        else
            echo -e "${YELLOW}⚠️  Checksum verification failed. Proceeding anyway...${NC}"
        fi
    fi

    # Extract
    echo "📂 Extracting..."
    if [ "${ext}" = "tar.gz" ]; then
        tar -xzf "${temp_dir}/${archive_name}.${ext}" -C "${temp_dir}" || { rm -rf "${temp_dir}"; return 1; }
    else
        tar -xJf "${temp_dir}/${archive_name}.${ext}" -C "${temp_dir}" || { rm -rf "${temp_dir}"; return 1; }
    fi

    if [ ! -f "${temp_dir}/${BINARY}" ]; then
        echo -e "${RED}❌ Binary not found in archive.${NC}"
        rm -rf "${temp_dir}"
        return 1
    fi

    # Install
    mkdir -p "${INSTALL_DIR}"
    echo "📥 Installing to ${INSTALL_DIR}/${BINARY}..."
    cp "${temp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    rm -rf "${temp_dir}"

    return 0
}

# ── Fallback: cargo install ──────────────────────────────────────────
install_via_cargo() {
    if ! have_cmd cargo; then
        echo -e "${RED}❌ Rust/cargo not installed.${NC}"
        echo "   Install Rust: https://rustup.rs/"
        echo "   Or download a release binary manually from:"
        echo "   https://github.com/${REPO}/releases"
        exit 1
    fi

    echo "🔨 Installing via cargo (this may take a few minutes)..."
    cargo install "${BINARY}" --root "${HOME}/.local"
}

# ── Main install flow ─────────────────────────────────────────────────
if install_from_release "${VERSION}"; then
    :
else
    echo ""
    install_via_cargo
fi

# ── Verify and report ────────────────────────────────────────────────
echo ""
echo -e "${CYAN}══════════════════════════════════════════════${NC}"
if have_cmd "${BINARY}"; then
    echo -e "${GREEN}  ✅ Git Hero installed successfully!${NC}"
    echo -e "${CYAN}══════════════════════════════════════════════${NC}"
    echo ""
    echo "  Run: ${GREEN}${BINARY}${NC}"
else
    if [ -x "${INSTALL_DIR}/${BINARY}" ]; then
        echo -e "${GREEN}  ✅ Git Hero installed at ${INSTALL_DIR}/${BINARY}${NC}"
        echo -e "${CYAN}══════════════════════════════════════════════${NC}"
        echo ""
        if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
            echo -e "${YELLOW}💡 Add ${INSTALL_DIR} to your PATH:${NC}"
            echo "   echo 'export PATH=\"\${HOME}/.local/bin:\${PATH}\"' >> ~/.zshrc"
            echo "   source ~/.zshrc"
            echo ""
        fi
        echo "  Run: ${BINARY}"
    else
        echo -e "${RED}  ❌ Installation may have failed.${NC}"
        echo -e "${CYAN}══════════════════════════════════════════════${NC}"
        echo "  Check the output above for errors."
        exit 1
    fi
fi
