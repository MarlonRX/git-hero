#!/bin/bash
# ============================================================================
# Git Hero - Full Deployment Pipeline
# Deploys to: GitHub Releases, crates.io, Homebrew, AUR, Snap
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
REPO="MarlonRX/git-hero"
REPO_URL="https://github.com/${REPO}"
APP_NAME="gith"

# ── Helper functions ─────────────────────────────────────────────────────
status_ok()   { echo -e "  ${GREEN}✔${NC}  $1"; }
status_warn() { echo -e "  ${YELLOW}⚠${NC}  $1"; }
status_fail() { echo -e "  ${RED}✘${NC}  $1"; }
step_header() { echo -e "\n${BLUE}${BOLD}[$1/$2]${NC} ${WHITE}$3${NC}\n${DIM}$(printf '%.0s─' {1..56})${NC}"; }

# ── Banner ───────────────────────────────────────────────────────────────
echo ""
echo -e "${BLUE}${BOLD}"
echo "  ╔══════════════════════════════════════════════════════╗"
echo "  ║                                                      ║"
echo "  ║          Git Hero · Deployment Pipeline              ║"
echo "  ║                                                      ║"
echo "  ╚══════════════════════════════════════════════════════╝"
echo -ne "${NC}"
echo ""

# ── Pre-checks ──────────────────────────────────────────────────────────
echo -e "${WHITE}${BOLD}Pre-flight checks${NC}"
echo -e "${DIM}$(printf '%.0s─' {1..56})${NC}"

# Git
if ! command -v git &> /dev/null; then
    status_fail "git not found"
    exit 1
fi

# Cargo
if ! command -v cargo &> /dev/null; then
    status_fail "cargo not found"
    exit 1
fi

status_ok "git, cargo"

# gh CLI (optional)
HAS_GH=false
if command -v gh &> /dev/null; then
    HAS_GH=true
    status_ok "gh CLI"
else
    status_warn "gh CLI not available"
fi

# snapcraft (optional)
HAS_SNAPCRAFT=false
if command -v snapcraft &> /dev/null; then
    HAS_SNAPCRAFT=true
    status_ok "snapcraft"
else
    status_warn "snapcraft not available (Snap will be skipped)"
fi

# makepkg (optional, AUR)
HAS_MAKEPKG=false
if command -v makepkg &> /dev/null; then
    HAS_MAKEPKG=true
    status_ok "makepkg"
else
    status_warn "makepkg not available (AUR will be skipped)"
fi

# ── Version ─────────────────────────────────────────────────────────────
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
TAG="v${VERSION}"
echo ""
echo -e "  ${DIM}Version${NC}  ${WHITE}${BOLD}${VERSION}${NC}"
echo -e "  ${DIM}Tag${NC}      ${WHITE}${BOLD}${TAG}${NC}"

# ── Confirmation ─────────────────────────────────────────────────────────
echo ""
read -p "  Continue with deployment? (y/N): " confirm
if [[ ! "$confirm" =~ ^[yY]$ ]]; then
    echo -e "\n  ${DIM}Cancelled.${NC}"
    exit 0
fi

# ── 1. Build cross-platform ─────────────────────────────────────────────
step_header 1 7 "Cross-platform build"
./scripts/release.sh "$VERSION"
status_ok "Binaries compiled"

# ── 2. Create tag and push ──────────────────────────────────────────────
step_header 2 7 "Git tag ${TAG}"
git tag -a "$TAG" -m "Release $TAG" 2>/dev/null || {
    status_warn "Tag already exists, updating..."
    git tag -d "$TAG"
    git tag -a "$TAG" -m "Release $TAG"
}
git push origin "$TAG"
status_ok "Tag created and pushed"

# ── 3. Create GitHub Release ────────────────────────────────────────────
step_header 3 7 "GitHub Release"
if [ "$HAS_GH" = true ]; then
    gh release create "$TAG" \
        target/release-artifacts/*.tar.gz \
        target/release-artifacts/*.zip \
        target/release-artifacts/checksums.txt \
        --title "gith $VERSION" \
        --notes "Release $TAG of gith" \
        --draft
    status_ok "Release created (draft)"
    echo -e "  ${DIM}Review:${NC} gh release view $TAG --web"
else
    status_warn "gh CLI not available"
    echo -e "  ${DIM}Create the release manually with files in target/release-artifacts/${NC}"
fi

# ── 4. Publish to crates.io ─────────────────────────────────────────────
step_header 4 7 "crates.io"
read -p "  Publish to crates.io? (y/N): " confirm_crate
if [[ "$confirm_crate" =~ ^[yY]$ ]]; then
    cargo publish
    status_ok "Published to crates.io"
else
    echo -e "  ${DIM}○  Skipped${NC}"
fi

# ── 5. Update Homebrew tap ──────────────────────────────────────────────
step_header 5 7 "Homebrew tap"
if [ -d "homebrew-tap" ]; then
    ./scripts/update-homebrew.sh
    read -p "  Push to Homebrew tap? (y/N): " confirm_brew
    if [[ "$confirm_brew" =~ ^[yY]$ ]]; then
        cd homebrew-tap
        git add git-hero.rb
        git commit -m "Update $APP_NAME to $VERSION"
        git push
        status_ok "Tap updated"
        cd ..
    else
        echo -e "  ${DIM}○  Not pushed, do it manually${NC}"
    fi
else
    status_warn "homebrew-tap directory not found"
    echo -e "  ${DIM}Create your tap at: https://github.com/MarlonRX/homebrew-git-hero${NC}"
fi

# ── 6. Update AUR ──────────────────────────────────────────────────────
step_header 6 7 "AUR package"
if [ "$HAS_MAKEPKG" = true ] && [ -d "aur" ]; then
    ./scripts/update-aur.sh
    read -p "  Push to AUR? (y/N): " confirm_aur
    if [[ "$confirm_aur" =~ ^[yY]$ ]]; then
        cd aur
        makepkg --printsrcinfo > .SRCINFO
        git add PKGBUILD .SRCINFO
        git commit -m "Update $APP_NAME to $VERSION"
        git push
        status_ok "AUR updated"
        cd ..
    else
        echo -e "  ${DIM}○  Not pushed, do it manually${NC}"
    fi
else
    status_warn "makepkg or aur directory not available"
fi

# ── 7. Build Snap ──────────────────────────────────────────────────────
step_header 7 7 "Snap package"
if [ "$HAS_SNAPCRAFT" = true ]; then
    cd snap
    snapcraft
    read -p "  Upload to Snap Store? (y/N): " confirm_snap
    if [[ "$confirm_snap" =~ ^[yY]$ ]]; then
        snapcraft upload --release=stable *.snap
        status_ok "Snap published"
    else
        echo -e "  ${DIM}○  Not published${NC}"
    fi
    cd ..
else
    status_warn "snapcraft not installed, skipping"
fi

# ── Summary ─────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}${BOLD}"
echo "  ╔══════════════════════════════════════════════════════╗"
echo "  ║                                                      ║"
echo "  ║          ✔  Deployment Complete                      ║"
echo "  ║                                                      ║"
echo "  ╚══════════════════════════════════════════════════════╝"
echo -ne "${NC}"
echo ""
echo -e "  ${WHITE}${BOLD}Deployment summary for v${VERSION}:${NC}"
echo ""
echo -e "  ${GREEN}✔${NC}  Git tag              ${DIM}${TAG}${NC}"
$HAS_GH && echo -e "  ${GREEN}✔${NC}  GitHub Release       ${DIM}${REPO_URL}/releases/tag/${TAG}${NC}"
[[ "${confirm_crate:-}" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✔${NC}  crates.io            ${DIM}https://crates.io/crates/${APP_NAME}${NC}"
[ -d "homebrew-tap" ] && [[ "${confirm_brew:-}" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✔${NC}  Homebrew tap         ${DIM}https://github.com/MarlonRX/homebrew-git-hero${NC}"
[ "$HAS_MAKEPKG" = true ] && [[ "${confirm_aur:-}" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✔${NC}  AUR                  ${DIM}https://aur.archlinux.org/packages/${APP_NAME}${NC}"
[ "$HAS_SNAPCRAFT" = true ] && [[ "${confirm_snap:-}" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✔${NC}  Snap Store           ${DIM}https://snapcraft.io/${APP_NAME}${NC}"
echo ""
