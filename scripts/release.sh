#!/bin/bash
# ============================================================================
# Git Hero - Release Script
# Compiles binaries for multiple platforms and creates release artifacts
# ============================================================================
set -euo pipefail

# ── Colors & Styles ──────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# ── Configuration ────────────────────────────────────────────────────────
APP_NAME="git-hero"
VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"
RELEASE_DIR="target/release-artifacts"
BINARY_NAME="$APP_NAME"

# ── Banner ───────────────────────────────────────────────────────────────
echo ""
echo -e "${BLUE}${BOLD}"
echo "  ╔══════════════════════════════════════════════════════╗"
echo "  ║                                                      ║"
printf "  ║   Git Hero · Release Builder v%-23s ║\n" "${VERSION}"
echo "  ║                                                      ║"
echo "  ╚══════════════════════════════════════════════════════╝"
echo -ne "${NC}"
echo ""

# ── Verify version ──────────────────────────────────────────────────────
if [ -z "$VERSION" ]; then
    echo -e "  ${RED}✘  Error: could not determine version${NC}"
    echo -e "     Usage: $0 [version]"
    echo -e "     Example: $0 0.1.0"
    exit 1
fi

# ── Clean previous builds ───────────────────────────────────────────────
echo -e "  ${DIM}Cleaning previous builds...${NC}"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"
cargo clean 2>/dev/null || true

# ── Targets to compile ──────────────────────────────────────────────────
TARGETS=(
    "x86_64-unknown-linux-gnu:linux:x86_64:gnu"
    "x86_64-unknown-linux-musl:linux:x86_64:musl"
    "aarch64-unknown-linux-gnu:linux:aarch64:gnu"
    "x86_64-apple-darwin:macos:x86_64:darwin"
    "aarch64-apple-darwin:macos:aarch64:darwin"
    "x86_64-pc-windows-msvc:windows:x86_64:msvc.exe"
)

total=${#TARGETS[@]}
current=0

echo ""
echo -e "  ${WHITE}${BOLD}Building for ${total} targets${NC}"
echo -e "  ${DIM}$(printf '%.0s─' {1..50})${NC}"

# ── Compile for each target ──────────────────────────────────────────────
for target_info in "${TARGETS[@]}"; do
    IFS=':' read -ra parts <<< "$target_info"
    target="${parts[0]}"
    os="${parts[1]}"
    arch="${parts[2]}"
    suffix="${parts[3]}"
    current=$((current + 1))

    echo ""
    echo -e "  ${BLUE}${BOLD}[${current}/${total}]${NC} ${WHITE}${os} (${arch})${NC} ${DIM}${target}${NC}"

    # Check if target is installed
    if ! rustup target list --installed | grep -q "$target"; then
        echo -e "       Installing target..."
        rustup target add "$target" 2>/dev/null || {
            echo -e "  ${YELLOW}⚠${NC}  Could not install target, skipping"
            continue
        }
    fi

    # Compile
    if cargo build --release --target "$target" 2>/dev/null; then
        # Determine binary extension
        if [[ "$target" == *"windows"* ]]; then
            binary_ext=".exe"
        else
            binary_ext=""
        fi

        binary_path="target/$target/release/${BINARY_NAME}${binary_ext}"
        if [ -f "$binary_path" ]; then
            # Create release directory
            artifact_dir="$RELEASE_DIR/${APP_NAME}-${VERSION}-${os}-${arch}"
            mkdir -p "$artifact_dir"

            # Copy binary
            cp "$binary_path" "$artifact_dir/${BINARY_NAME}${binary_ext}"
            chmod +x "$artifact_dir/${BINARY_NAME}${binary_ext}"

            # Copy documentation
            cp README.md "$artifact_dir/"
            cp LICENSE "$artifact_dir/" 2>/dev/null || true

            # Create compressed archive
            cd "$RELEASE_DIR"
            if [[ "$target" == *"windows"* ]]; then
                zip -qr "${artifact_dir}.zip" "${artifact_dir##*/}"
                echo -e "  ${GREEN}✔${NC}  ${artifact_dir##*/}.zip"
            else
                tar -czf "${artifact_dir}.tar.gz" "${artifact_dir##*/}"
                echo -e "  ${GREEN}✔${NC}  ${artifact_dir##*/}.tar.gz"
            fi
            cd - > /dev/null
        else
            echo -e "  ${RED}✘${NC}  Binary not found: $binary_path"
        fi
    else
        echo -e "  ${RED}✘${NC}  Compilation failed for $target"
    fi
done

# ── Generate checksums ──────────────────────────────────────────────────
echo ""
echo -e "  ${WHITE}${BOLD}Generating checksums${NC}"
echo -e "  ${DIM}$(printf '%.0s─' {1..50})${NC}"
cd "$RELEASE_DIR"
for file in *.tar.gz *.zip; do
    if [ -f "$file" ]; then
        shasum -a 256 "$file" >> checksums.txt
    fi
done
echo ""
cat checksums.txt | while read -r line; do
    echo -e "  ${DIM}${line}${NC}"
done
cd - > /dev/null

# ── Summary ─────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}"
echo "  ╔══════════════════════════════════════════════════════╗"
echo "  ║                                                      ║"
echo "  ║          ✔  Release build complete                   ║"
echo "  ║                                                      ║"
echo "  ╚══════════════════════════════════════════════════════╝"
echo -ne "${NC}"
echo ""
echo -e "  ${DIM}Artifacts:${NC}  $RELEASE_DIR/"
ls -lh "$RELEASE_DIR"/ | grep -E '\.(tar\.gz|zip|txt)$' | while read -r line; do
    echo -e "  ${DIM}  ${line}${NC}"
done || true
echo ""
echo -e "  ${WHITE}${BOLD}Next steps:${NC}"
echo -e "  ${DIM}1.${NC} Create tag:    ${CYAN}git tag -a v$VERSION -m 'Release v$VERSION'${NC}"
echo -e "  ${DIM}2.${NC} Push tag:      ${CYAN}git push origin v$VERSION${NC}"
echo -e "  ${DIM}3.${NC} GitHub release: ${CYAN}gh release create v$VERSION $RELEASE_DIR/*${NC}"
echo -e "  ${DIM}4.${NC} crates.io:     ${CYAN}cargo publish${NC}"
echo -e "  ${DIM}5.${NC} Homebrew:       Update tap with SHA256"
echo ""
