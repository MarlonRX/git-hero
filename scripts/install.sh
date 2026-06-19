#!/bin/sh
# ── Git Hero Installer (gith binary) ────────────────────────────────
# Minimal POSIX-compatible install script.
#
# Strategy: try cargo install first (fast), fall back to build from
# source (always works). Removes old `git-hero` binary if present.

set -u

BINARY="gith"
OLD_BINARY="git-hero"
REPO="MarlonRX/git-hero"
INSTALL_DIR="${HOME}/.local/bin"

# ── ANSI colors (works on any terminal) ─────────────────────────────
BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Use printf for portable colored output (echo -e is not POSIX)
info()   { printf "${CYAN}${BOLD}  %s${NC}\n" "$1"; }
ok()     { printf "  ${GREEN}%s${NC}\n" "$1"; }
warn()   { printf "  ${YELLOW}%s${NC}\n" "$1"; }
fail()   { printf "  ${RED}%s${NC}\n" "$1"; }
step()   { printf "  ${BOLD}%s${NC}  %s\n" "$1" "$2"; }

have_cmd() { command -v "$1" >/dev/null 2>&1; }

info "Git Hero installer"
printf "  ${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
echo ""

# ── Migrate old git-hero binary ──────────────────────────────────────
if [ -x "${INSTALL_DIR}/${OLD_BINARY}" ]; then
    warn "Removing old '${OLD_BINARY}' binary..."
    rm -f "${INSTALL_DIR}/${OLD_BINARY}"
fi

# ── 1. Install via cargo ────────────────────────────────────────────
if have_cmd cargo; then
    step "➜" "Installing via cargo..."
    if cargo install "${BINARY}" --root "${HOME}/.local" 2>/dev/null; then
        ok "done"
        INSTALLED=1
    else
        warn "not published on crates.io — building from source"
    fi
fi

# ── 2. Build from source ────────────────────────────────────────────
if [ -z "${INSTALLED:-}" ]; then
    if ! have_cmd cargo; then
        echo ""
        fail "Rust/cargo required. Install it first:"
        printf "     ${CYAN}https://rustup.rs/${NC}\n"
        echo ""
        exit 1
    fi

    if ! have_cmd git; then
        fail "git required. Install it first."
        exit 1
    fi

    BUILD_DIR=$(mktemp -d)
    step "➜" "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "${BUILD_DIR}/gith" >/dev/null 2>&1 || {
        fail "Clone failed"
        rm -rf "${BUILD_DIR}"
        exit 1
    }

    step "➜" "Building from source (may take a few minutes)..."
    (cd "${BUILD_DIR}/gith" && cargo build --release) >/dev/null 2>&1
    BUILD_EXIT=$?

    if [ "$BUILD_EXIT" -ne 0 ]; then
        fail "Build failed. Try manually:"
        echo "     git clone https://github.com/${REPO}.git"
        echo "     cd gith && cargo build --release"
        rm -rf "${BUILD_DIR}"
        exit 1
    fi

    mkdir -p "${INSTALL_DIR}"
    cp "${BUILD_DIR}/gith/target/release/gith" "${INSTALL_DIR}/gith"
    chmod +x "${INSTALL_DIR}/gith"
    rm -rf "${BUILD_DIR}"
    ok "✔  Built and installed"
fi

# ── Symlink for backwards compatibility ──────────────────────────────
if [ -x "${INSTALL_DIR}/${BINARY}" ] && ! have_cmd "${OLD_BINARY}"; then
    ln -sf "${INSTALL_DIR}/${BINARY}" "${INSTALL_DIR}/${OLD_BINARY}" 2>/dev/null || true
fi

# ── Verify ──────────────────────────────────────────────────────────
echo ""
if have_cmd "${BINARY}"; then
    ok "✔  ${BOLD}gith${NC} ${GREEN}installed successfully!${NC}"
    echo ""
    echo "  Get started:"
    printf "    ${CYAN}${BOLD}gith${NC}\n"
elif [ -x "${INSTALL_DIR}/${BINARY}" ]; then
    ok "✔  ${BOLD}gith${NC} ${GREEN}installed!${NC}"
    echo ""
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        warn "Add to PATH:"
        echo '     export PATH="${HOME}/.local/bin:${PATH}"'
        echo ""
    fi
    echo "  Get started:"
    printf "    ${CYAN}${BOLD}gith${NC}\n"
else
    fail "✘  Installation failed."
    exit 1
fi
echo ""
