#!/bin/bash
# ============================================================================
# Git Hero - Full Deployment Script
# Despliega a: Cargo, Homebrew, AUR, Snap
# ============================================================================
set -euo pipefail

# ── Colores ───────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ── Configuración ─────────────────────────────────────────────────────────
REPO="MarlonRX/git-hero"
REPO_URL="https://github.com/${REPO}"
APP_NAME="git-hero"

# ── Banner ────────────────────────────────────────────────────────────────
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              git-hero Deployment Pipeline                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# ── Pre-checks ───────────────────────────────────────────────────────────
echo -e "${YELLOW}→ Verificando requisitos...${NC}"

# Git
if ! command -v git &> /dev/null; then
    echo -e "${RED}✗ git no encontrado${NC}"
    exit 1
fi

# Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ cargo no encontrado${NC}"
    exit 1
fi

# gh CLI (opcional)
HAS_GH=false
if command -v gh &> /dev/null; then
    HAS_GH=true
fi

# snapcraft (opcional)
HAS_SNAPCRAFT=false
if command -v snapcraft &> /dev/null; then
    HAS_SNAPCRAFT=true
fi

# makepkg (opcional, AUR)
HAS_MAKEPKG=false
if command -v makepkg &> /dev/null; then
    HAS_MAKEPKG=true
fi

echo -e "${GREEN}✓ git, cargo${NC}"
$HAS_GH && echo -e "${GREEN}✓ gh CLI${NC}" || echo -e "${YELLOW}⚠ gh CLI no disponible${NC}"
$HAS_SNAPCRAFT && echo -e "${GREEN}✓ snapcraft${NC}" || echo -e "${YELLOW}⚠ snapcraft no disponible (Snap se saltará)${NC}"
$HAS_MAKEPKG && echo -e "${GREEN}✓ makepkg${NC}" || echo -e "${YELLOW}⚠ makepkg no disponible (AUR se saltará)${NC}"

# ── Versión ──────────────────────────────────────────────────────────────
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
TAG="v${VERSION}"
echo ""
echo -e "${YELLOW}→ Versión detectada: ${VERSION}${NC}"
echo -e "${YELLOW}→ Tag: ${TAG}${NC}"
echo ""

# ── Confirmación ─────────────────────────────────────────────────────────
read -p "¿Continuar con el despliegue? (y/N): " confirm
if [[ ! "$confirm" =~ ^[yY]$ ]]; then
    echo "Cancelado."
    exit 0
fi
echo ""

# ── 1. Build cross-platform ───────────────────────────────────────────────
echo -e "${BLUE}[1/6] Compilando binarios cross-platform...${NC}"
./scripts/release.sh "$VERSION"
echo -e "${GREEN}✓ Binarios compilados${NC}"
echo ""

# ── 2. Crear tag y push ──────────────────────────────────────────────────
echo -e "${BLUE}[2/6] Creando tag ${TAG}...${NC}"
git tag -a "$TAG" -m "Release $TAG" 2>/dev/null || {
    echo -e "${YELLOW}⚠ Tag ya existe, actualizando...${NC}"
    git tag -d "$TAG"
    git tag -a "$TAG" -m "Release $TAG"
}
git push origin "$TAG"
echo -e "${GREEN}✓ Tag creado y pusheado${NC}"
echo ""

# ── 3. Crear GitHub Release ─────────────────────────────────────────────
echo -e "${BLUE}[3/6] Creando GitHub Release...${NC}"
if [ "$HAS_GH" = true ]; then
    gh release create "$TAG" \
        target/release-artifacts/*.tar.gz \
        target/release-artifacts/*.zip \
        target/release-artifacts/checksums.txt \
        --title "git-hero $VERSION" \
        --notes "Release $TAG of git-hero" \
        --draft
    echo -e "${GREEN}✓ Release creado (borrador)${NC}"
    echo "  → Revisa y publica: gh release view $TAG --web"
else
    echo -e "${YELLOW}⚠ gh CLI no disponible${NC}"
    echo "  → Crea el release manualmente con los archivos en target/release-artifacts/"
fi
echo ""

# ── 4. Publicar en crates.io ─────────────────────────────────────────────
echo -e "${BLUE}[4/6] Publicando en crates.io...${NC}"
read -p "¿Publicar en crates.io? (y/N): " confirm_crate
if [[ "$confirm_crate" =~ ^[yY]$ ]]; then
    cargo publish
    echo -e "${GREEN}✓ Publicado en crates.io${NC}"
else
    echo -e "${YELLOW}⊘ Saltado crates.io${NC}"
fi
echo ""

# ── 5. Actualizar Homebrew tap ───────────────────────────────────────────
echo -e "${BLUE}[5/6] Actualizando Homebrew tap...${NC}"
if [ -d "homebrew-tap" ]; then
    ./scripts/update-homebrew.sh
    echo ""
    read -p "¿Push al tap de Homebrew? (y/N): " confirm_brew
    if [[ "$confirm_brew" =~ ^[yY]$ ]]; then
        cd homebrew-tap
        git add git-hero.rb
        git commit -m "Update $APP_NAME to $VERSION"
        git push
        echo -e "${GREEN}✓ Tap actualizado${NC}"
        cd ..
    else
        echo -e "${YELLOW}⊘ No pusheado, hazlo manualmente${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Directorio homebrew-tap no existe${NC}"
    echo "  → Crea tu tap en: https://github.com/$REPO_USERNAME/homebrew-tap"
fi
echo ""

# ── 6. Actualizar AUR ────────────────────────────────────────────────────
echo -e "${BLUE}[6/7] Actualizando AUR...${NC}"
if [ "$HAS_MAKEPKG" = true ] && [ -d "aur" ]; then
    ./scripts/update-aur.sh
    echo ""
    read -p "¿Push al AUR? (y/N): " confirm_aur
    if [[ "$confirm_aur" =~ ^[yY]$ ]]; then
        cd aur
        makepkg --printsrcinfo > .SRCINFO
        git add PKGBUILD .SRCINFO
        git commit -m "Update $APP_NAME to $VERSION"
        git push
        echo -e "${GREEN}✓ AUR actualizado${NC}"
        cd ..
    else
        echo -e "${YELLOW}⊘ No pusheado, hazlo manualmente${NC}"
    fi
else
    echo -e "${YELLOW}⚠ makepkg o directorio AUR no disponible${NC}"
fi
echo ""

# ── 7. Build Snap ────────────────────────────────────────────────────────
echo -e "${BLUE}[7/7] Construyendo Snap...${NC}"
if [ "$HAS_SNAPCRAFT" = true ]; then
    cd snap
    snapcraft
    echo ""
    read -p "¿Subir a Snap Store? (y/N): " confirm_snap
    if [[ "$confirm_snap" =~ ^[yY]$ ]]; then
        snapcraft upload --release=stable *.snap
        echo -e "${GREEN}✓ Snap publicado${NC}"
    else
        echo -e "${YELLOW}⊘ No publicado${NC}"
    fi
    cd ..
else
    echo -e "${YELLOW}⚠ snapcraft no instalado, saltando${NC}"
fi
echo ""

# ── Resumen ──────────────────────────────────────────────────────────────
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              Deployment Complete!                          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Resumen del despliegue v${VERSION}:"
echo ""
echo -e "  ${GREEN}✓${NC} Tag creado:              $TAG"
$HAS_GH && echo -e "  ${GREEN}✓${NC} GitHub Release:          https://github.com/$REPO/releases/tag/$TAG"
[[ "$confirm_crate" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✓${NC} crates.io:               https://crates.io/crates/$APP_NAME"
[ -d "homebrew-tap" ] && [[ "$confirm_brew" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✓${NC} Homebrew tap:            https://github.com/$REPO_USERNAME/homebrew-tap"
[ "$HAS_MAKEPKG" = true ] && [[ "$confirm_aur" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✓${NC} AUR:                     https://aur.archlinux.org/packages/$APP_NAME"
[ "$HAS_SNAPCRAFT" = true ] && [[ "$confirm_snap" =~ ^[yY]$ ]] && echo -e "  ${GREEN}✓${NC} Snap Store:              https://snapcraft.io/$APP_NAME"
echo ""
