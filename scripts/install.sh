#!/usr/bin/env bash
# ── Git Hero Installer ───────────────────────────────────────────────
# One-liner install script for end users
# Usage: curl -fsSL https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.sh | bash
#
# Or locally: ./scripts/install.sh [version]
#
# Strategy:
#   1. Try to install from prebuilt release binary (fastest)
#   2. Fall back to `cargo install` from crates.io (fast, if published)
#   3. Fall back to git clone + cargo build from source (always works)

set -u  # NOT -e: we handle errors explicitly

VERSION="${1:-latest}"
BINARY="git-hero"
REPO="MarlonRX/git-hero"
INSTALL_DIR="${HOME}/.local/bin"

# ── Colors & Styles ──────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# ── Helper functions ─────────────────────────────────────────────────
have_cmd() { command -v "$1" >/dev/null 2>&1; }

curl_get() {
    curl -fsSL --max-time 30 "$1" -o "$2" 2>/dev/null
}

# Spinner for long-running operations with 8‑bit robot sprite
spinner() {
    local pid=$1
    local label="${2:-Working...}"
    # Simple robot ASCII frames (5)
    local robot_frames=('🤖' '⡿' '⣟' '⣯' '⣿')
    local spinner_frames=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    local i=0
    local r=0
    local start_time=$(date +%s)

    # Hide cursor
    tput civis 2>/dev/null || true

    while kill -0 "$pid" 2>/dev/null; do
        local elapsed=$(( $(date +%s) - start_time ))
        printf "\r  ${CYAN}${spinner_frames[$i]}${NC} ${label} ${DIM}(%ds)${NC} ${MAGENTA}%s${NC} " "$elapsed" "${robot_frames[$r]}"
        i=$(( (i + 1) % ${#spinner_frames[@]} ))
        r=$(( (r + 1) % ${#robot_frames[@]} ))
        sleep 0.1
    done

    # Show cursor
    tput cnorm 2>/dev/null || true

    wait "$pid"
    return $?
}

# Print a step header
step() {
    local num="$1"
    local total="$2"
    local label="$3"
    echo ""
    echo -e "  ${BLUE}${BOLD}[${num}/${total}]${NC} ${WHITE}${label}${NC}"
    echo -e "  ${DIM}$(printf '%.0s─' {1..44})${NC}"
}

# Print status result
status_ok()   { printf "\r  ${GREEN}✔${NC}  %s\n" "$1"; }
status_warn() { printf "\r  ${YELLOW}⚠${NC}  %s\n" "$1"; }
status_fail() { printf "\r  ${RED}✘${NC}  %s\n" "$1"; }
status_skip() { printf "\r  ${DIM}○${NC}  ${DIM}%s${NC}\n" "$1"; }

# ── Banner ───────────────────────────────────────────────────────────
# Print a header with app name and version
print_app_header() {
    echo -e "${MAGENTA}${BOLD}GIT HERO V beta 0.1${NC}"
}
echo ""
print_app_header
echo -e "${CYAN}${BOLD}"
cat << 'LOGO'
      ┌─────────────────────────────────────────┐
      │                                         │
      │    ╔═╗ ╦ ╔╦╗  ╦ ╦ ╔═╗ ╦═╗ ╔═╗         │
      │    ║ ╦ ║  ║   ╠═╣ ║╣  ╠╦╝ ║ ║         │
      │    ╚═╝ ╩  ╩   ╩ ╩ ╚═╝ ╩╚═ ╚═╝         │
      │                                         │
      └─────────────────────────────────────────┘
LOGO
echo -ne "${NC}"
echo -e "  ${DIM}Fast & visual TUI for Git${NC}"
echo -e "  ${DIM}https://github.com/${REPO}${NC}"
echo ""

# ── Detect platform ──────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)  PLATFORM="linux" ;;
    Darwin) PLATFORM="macos" ;;
    *)
        echo -e "  ${RED}✘  Unsupported OS: ${OS}${NC}"
        echo -e "     Git Hero currently supports Linux and macOS."
        exit 1
        ;;
esac

case "${ARCH}" in
    x86_64)       ARCH_NAME="x86_64" ;;
    arm64|aarch64) ARCH_NAME="arm64" ;;
    *)
        echo -e "  ${RED}✘  Unsupported architecture: ${ARCH}${NC}"
        exit 1
        ;;
esac

echo -e "  ${DIM}Platform${NC}  ${WHITE}${PLATFORM}-${ARCH_NAME}${NC}"
echo -e "  ${DIM}Target${NC}    ${WHITE}${INSTALL_DIR}${NC}"

# ── Try prebuilt release ─────────────────────────────────────────────
install_from_release() {
    local version="$1"

    if [ "${version}" = "latest" ]; then
        echo -e "       Fetching latest version from GitHub..."
        local api_resp
        api_resp=$(curl -fsSL --max-time 15 "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null) || true

        if [ -n "${api_resp}" ]; then
            version=$(echo "${api_resp}" | grep '"tag_name":' | head -1 | sed -E 's/.*"v?([^"]+)".*/\1/')
        fi

        if [ -z "${version}" ] || [ "${version}" = "latest" ]; then
            status_warn "No releases found (API rate limit or no releases yet)"
            return 1
        fi
    fi

    echo -e "       Version: ${GREEN}v${version}${NC}"

    local archive_name="${BINARY}-v${version}-${PLATFORM}-${ARCH_NAME}"
    local ext="tar.gz"
    [ "${PLATFORM}" = "linux" ] && ext="tar.xz"

    local download_url="https://github.com/${REPO}/releases/download/v${version}/${archive_name}.${ext}"
    local temp_dir
    temp_dir=$(mktemp -d)

    echo -e "       Downloading ${DIM}${archive_name}.${ext}${NC}..."
    if ! curl_get "${download_url}" "${temp_dir}/${archive_name}.${ext}"; then
        status_warn "Binary not available for ${PLATFORM}-${ARCH_NAME}"
        rm -rf "${temp_dir}"
        return 1
    fi

    # Optional checksum verification
    local checksum_url="${download_url}.sha256"
    if curl_get "${checksum_url}" "${temp_dir}/checksum.txt"; then
        if (cd "${temp_dir}" && shasum -a 256 -c checksum.txt >/dev/null 2>&1); then
            echo -e "       ${GREEN}🔐 Checksum verified${NC}"
        else
            echo -e "       ${YELLOW}⚠ Checksum mismatch (proceeding)${NC}"
        fi
    fi

    # Extract
    echo -e "       Extracting..."
    if [ "${ext}" = "tar.gz" ]; then
        tar -xzf "${temp_dir}/${archive_name}.${ext}" -C "${temp_dir}" || { rm -rf "${temp_dir}"; return 1; }
    else
        tar -xJf "${temp_dir}/${archive_name}.${ext}" -C "${temp_dir}" || { rm -rf "${temp_dir}"; return 1; }
    fi

    if [ ! -f "${temp_dir}/${BINARY}" ]; then
        status_fail "Binary not found in archive"
        rm -rf "${temp_dir}"
        return 1
    fi

    # Install
    mkdir -p "${INSTALL_DIR}"
    cp "${temp_dir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    rm -rf "${temp_dir}"

    status_ok "Installed from prebuilt release"
    return 0
}

# ── Fallback 1: cargo install from crates.io ─────────────────────────
install_via_cargo_registry() {
    if ! have_cmd cargo; then
        status_skip "cargo not found, skipping"
        return 1
    fi

    echo -e "       Installing via ${DIM}cargo install${NC}..."
    if cargo install "${BINARY}" --root "${HOME}/.local" 2>/dev/null; then
        status_ok "Installed from crates.io"
        return 0
    else
        status_warn "Not published on crates.io yet"
        return 1
    fi
}

# ── Fallback 2: build from source ────────────────────────────────────
install_from_source() {
    if ! have_cmd cargo; then
        status_fail "Rust/cargo not installed"
        echo -e "       Install Rust: ${CYAN}https://rustup.rs/${NC}"
        return 1
    fi

    if ! have_cmd git; then
        status_fail "Git not installed"
        return 1
    fi

    local build_dir
    build_dir=$(mktemp -d)

    echo -e "       Cloning repository..."
    if ! git clone --depth 1 "https://github.com/${REPO}.git" "${build_dir}/git-hero" >/dev/null 2>&1; then
        status_fail "Could not clone repository"
        rm -rf "${build_dir}"
        return 1
    fi
    echo -e "       ${GREEN}✔${NC} Source cloned"

    # Build with spinner (suppress cargo output)
    (
        cd "${build_dir}/git-hero"
        cargo build --release >/dev/null 2>&1
    ) &
    local build_pid=$!

    spinner "$build_pid" "Compiling from source"
    local build_exit=$?

    if [ "$build_exit" -ne 0 ]; then
        echo ""
        status_fail "Build failed"
        echo -e "       ${DIM}Try building manually:${NC}"
        echo -e "       ${DIM}  git clone https://github.com/${REPO}.git${NC}"
        echo -e "       ${DIM}  cd git-hero && cargo build --release${NC}"
        rm -rf "${build_dir}"
        return 1
    fi
    echo ""

    mkdir -p "${INSTALL_DIR}"
    cp "${build_dir}/git-hero/target/release/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"
    rm -rf "${build_dir}"

    status_ok "Built and installed from source"
    return 0
}

# ── Main install flow ────────────────────────────────────────────────

step 1 3 "Prebuilt release binary"
if install_from_release "${VERSION}"; then
    INSTALL_METHOD="release"
else
    step 2 3 "crates.io registry"
    if install_via_cargo_registry; then
        INSTALL_METHOD="crates"
    else
        step 3 3 "Build from source"
        if install_from_source; then
            INSTALL_METHOD="source"
        else
            echo ""
            echo -e "  ${RED}${BOLD}╔══════════════════════════════════════════════╗${NC}"
            echo -e "  ${RED}${BOLD}║  ✘  All installation methods failed         ║${NC}"
            echo -e "  ${RED}${BOLD}╚══════════════════════════════════════════════╝${NC}"
            echo ""
            echo -e "  ${WHITE}Try one of these manual options:${NC}"
            echo ""
            echo -e "  ${DIM}1.${NC} Install Rust:  ${CYAN}https://rustup.rs/${NC}"
            echo -e "  ${DIM}2.${NC} Clone & build:"
            echo -e "     ${DIM}git clone https://github.com/${REPO}.git${NC}"
            echo -e "     ${DIM}cd git-hero && cargo build --release${NC}"
            echo -e "  ${DIM}3.${NC} Manual download:"
            echo -e "     ${DIM}https://github.com/${REPO}/releases${NC}"
            echo ""
            exit 1
        fi
    fi
fi

# ── Detect user's shell for PATH instructions ────────────────────────
detect_shell_config() {
    local shell_name
    shell_name="$(basename "${SHELL:-/bin/bash}")"
    case "$shell_name" in
        zsh)  echo "${HOME}/.zshrc" ;;
        bash)
            if [ -f "${HOME}/.bash_profile" ]; then
                echo "${HOME}/.bash_profile"
            else
                echo "${HOME}/.bashrc"
            fi
            ;;
        fish) echo "${HOME}/.config/fish/config.fish" ;;
        *)    echo "${HOME}/.profile" ;;
    esac
}

# ── Success banner ───────────────────────────────────────────────────
echo ""
echo -e "  ${GREEN}${BOLD}╔══════════════════════════════════════════════╗${NC}"
echo -e "  ${GREEN}${BOLD}║                                              ║${NC}"
echo -e "  ${GREEN}${BOLD}║   ✔  Git Hero installed successfully!        ║${NC}"
echo -e "  ${GREEN}${BOLD}║                                              ║${NC}"
echo -e "  ${GREEN}${BOLD}╚══════════════════════════════════════════════╝${NC}"
echo ""

if have_cmd "${BINARY}"; then
    echo -e "  ${WHITE}Get started:${NC}"
    echo -e "  ${DIM}$${NC} ${GREEN}git-hero${NC}"
    echo ""
elif [ -x "${INSTALL_DIR}/${BINARY}" ]; then
    if ! echo "${PATH}" | grep -q "${INSTALL_DIR}"; then
        shell_config="$(detect_shell_config)"
        shell_name="$(basename "${SHELL:-/bin/bash}")"

        echo -e "  ${YELLOW}${BOLD}⚠  Add to PATH${NC}"
        echo ""

        if [ "$shell_name" = "fish" ]; then
            echo -e "  ${DIM}$${NC} fish_add_path ${INSTALL_DIR}"
        else
            echo -e "  ${DIM}$${NC} echo 'export PATH=\"\${HOME}/.local/bin:\${PATH}\"' >> $(basename "$shell_config")"
            echo -e "  ${DIM}$${NC} source $(basename "$shell_config")"
        fi
        echo ""
    fi
    echo -e "  ${WHITE}Get started:${NC}"
    echo -e "  ${DIM}$${NC} ${GREEN}git-hero${NC}"
    echo ""
else
    echo -e "  ${RED}✘  Installation may have failed.${NC}"
    echo -e "     Check the output above for errors."
    exit 1
fi
