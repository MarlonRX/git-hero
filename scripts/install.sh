#!/bin/sh
# ── Git Hero Installer ───────────────────────────────────────────────
# Minimal POSIX-compatible install script.
#
# Strategy: try cargo install first (fast), fall back to build from
# source (always works). No fancy spinners, no API calls, no emoji.

set -u

BINARY="git-hero"
REPO="MarlonRX/git-hero"
INSTALL_DIR="${HOME}/.local/bin"

# ── Colors (ANSI, works on most terminals) ──────────────────────────
BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

have_cmd() { command -v "$1" >/dev/null 2>&1; }

echo ""
echo "${CYAN}${BOLD}  git-hero${NC} ${BOLD}installer${NC}"
echo "  ${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# ── 1. Install via cargo ────────────────────────────────────────────
if have_cmd cargo; then
    printf "  ${BOLD}➜${NC}  Installing via cargo..."
    if cargo install "${BINARY}" --root "${HOME}/.local" 2>/dev/null; then
        echo " ${GREEN}done${NC}"
        INSTALLED=1
    else
        echo " ${YELLOW}not published${NC}"
    fi
fi

# ── 2. Build from source ────────────────────────────────────────────
if [ -z "${INSTALLED:-}" ]; then
    if ! have_cmd cargo; then
        echo ""
        echo "  ${RED}✘  Rust/cargo required. Install it first:${NC}"
        echo "     ${CYAN}https://rustup.rs/${NC}"
        echo ""
        exit 1
    fi

    if ! have_cmd git; then
        echo "  ${RED}✘  git required. Install it first.${NC}"
        exit 1
    fi

    BUILD_DIR=$(mktemp -d)
    echo "  ${BOLD}➜${NC}  Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "${BUILD_DIR}/git-hero" >/dev/null 2>&1 || {
        echo "  ${RED}✘  Clone failed${NC}"
        rm -rf "${BUILD_DIR}"
        exit 1
    }

    echo "  ${BOLD}➜${NC}  Building from source (this may take a few minutes)..."
    (cd "${BUILD_DIR}/git-hero" && cargo build --release) >/dev/null 2>&1
    BUILD_EXIT=$?

    if [ "$BUILD_EXIT" -ne 0 ]; then
        echo "  ${RED}✘  Build failed. Try manually:${NC}"
        echo "     git clone https://github.com/${REPO}.git"
        echo "     cd git-hero && cargo build --release"
        rm -rf "${BUILD_DIR}"
        exit 1
    fi

    mkdir -p "${INSTALL_DIR}"
    cp "${BUILD_DIR}/git-hero/target/release/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    rm -rf "${BUILD_DIR}"
    echo "  ${GREEN}✔${NC}  Built and installed"
fi

# ── Verify ──────────────────────────────────────────────────────────
echo ""
if have_cmd "${BINARY}"; then
    echo "  ${GREEN}${BOLD}✔  Git Hero installed successfully!${NC}"
    echo ""
    echo "  Get started:"
    echo "    ${CYAN}${BOLD}git-hero${NC}"
elif [ -x "${INSTALL_DIR}/${BINARY}" ]; then
    echo "  ${GREEN}${BOLD}✔  Git Hero installed!${NC}"
    echo ""
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        echo "  ${YELLOW}⚠  Add to PATH:${NC}"
        echo "     export PATH=\"\${HOME}/.local/bin:\${PATH}\""
        echo ""
    fi
    echo "  Get started:"
    echo "    ${CYAN}${BOLD}git-hero${NC}"
else
    echo "  ${RED}✘  Installation failed.${NC}"
    exit 1
fi
echo ""
