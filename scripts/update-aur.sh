#!/bin/bash
# ============================================================================
# Git Hero - Update AUR PKGBUILD
# Automatically calculates the SHA256 of the release tarball and updates
# the PKGBUILD with the correct version and hash.
# ============================================================================
set -euo pipefail

# ── Colors ───────────────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

REPO="MarlonRX/git-hero"
AUR_DIR="aur"
PKGBUILD="$AUR_DIR/PKGBUILD"

echo ""
echo -e "${WHITE}${BOLD}AUR PKGBUILD Update${NC}"
echo -e "${DIM}$(printf '%.0s─' {1..50})${NC}"

# Detect latest version
if [ -d "$AUR_DIR/.git" ]; then
    cd "$AUR_DIR"
    latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")
    cd ..
else
    latest_tag="v0.1.0"
fi

echo -e "  ${DIM}Version:${NC}  $latest_tag"
tarball_url="https://github.com/$REPO/archive/refs/tags/$latest_tag.tar.gz"

# Calculate SHA256
echo -e "  ${DIM}Downloading tarball...${NC}"
sha256=$(curl -sL "$tarball_url" | shasum -a 256 | cut -d' ' -f1)
echo -e "  ${DIM}SHA256:${NC}   ${CYAN}${sha256}${NC}"

# Update PKGBUILD
echo -e "  ${DIM}Updating ${PKGBUILD}...${NC}"
version="${latest_tag#v}"
sed -i.bak \
    -e "s|^pkgver=.*|pkgver=$version|" \
    -e "s|sha256sums=('.*')|sha256sums=('$sha256')|" \
    "$PKGBUILD"

rm -f "$PKGBUILD.bak"

echo ""
echo -e "  ${GREEN}✔${NC}  PKGBUILD updated"
echo ""
echo -e "  ${WHITE}${BOLD}To verify:${NC}"
echo -e "  ${DIM}\$${NC} ${CYAN}cd $AUR_DIR && makepkg -si${NC}"
echo ""
echo -e "  ${WHITE}${BOLD}To publish to AUR:${NC}"
echo -e "  ${DIM}1.${NC} ${CYAN}cd $AUR_DIR${NC}"
echo -e "  ${DIM}2.${NC} ${CYAN}git add PKGBUILD .SRCINFO${NC}"
echo -e "  ${DIM}3.${NC} ${CYAN}git commit -m 'Update to $latest_tag'${NC}"
echo -e "  ${DIM}4.${NC} ${CYAN}git push${NC}"
echo ""
