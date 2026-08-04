#!/bin/bash
# ============================================================================
# Git Hero - Update Homebrew Formula
# Automatically calculates the SHA256 of the release tarball and updates
# the formula with the correct URL and hash.
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
TAP_DIR="homebrew-tap"
FORMULA="$TAP_DIR/git-hero.rb"

echo ""
echo -e "${WHITE}${BOLD}Homebrew Formula Update${NC}"
echo -e "${DIM}$(printf '%.0s─' {1..50})${NC}"

# Detect latest version
if [ -d "$TAP_DIR/.git" ]; then
    cd "$TAP_DIR"
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

# Update formula
echo -e "  ${DIM}Updating ${FORMULA}...${NC}"
version="${latest_tag#v}"
sed -i.bak \
    -e "s|url \".*\"|url \"https://github.com/$REPO/archive/refs/tags/$latest_tag.tar.gz\"|" \
    -e "s|sha256 \".*\"|sha256 \"$sha256\"|" \
    -e "s|v[0-9]*\.[0-9]*\.[0-9]*\.tar\.gz|v$version.tar.gz|g" \
    "$FORMULA"

rm -f "$FORMULA.bak"

echo ""
echo -e "  ${GREEN}✔${NC}  Formula updated"
echo ""
echo -e "  ${WHITE}${BOLD}Next steps:${NC}"
echo -e "  ${DIM}1.${NC} ${CYAN}cd $TAP_DIR${NC}"
echo -e "  ${DIM}2.${NC} ${CYAN}git add git-hero.rb${NC}"
echo -e "  ${DIM}3.${NC} ${CYAN}git commit -m 'Update git-hero to $latest_tag'${NC}"
echo -e "  ${DIM}4.${NC} ${CYAN}git push${NC}"
echo ""
