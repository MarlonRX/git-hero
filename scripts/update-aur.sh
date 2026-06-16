#!/bin/bash
# ============================================================================
# Update AUR PKGBUILD with SHA256
# Calcula automáticamente el SHA256 del tarball y actualiza el PKGBUILD
# ============================================================================
set -euo pipefail

REPO="MarlonRX/git-hero"
AUR_DIR="aur"
PKGBUILD="$AUR_DIR/PKGBUILD"

echo "→ Calculando SHA256 para AUR..."

# Detectar última versión
if [ -d "$AUR_DIR/.git" ]; then
    cd "$AUR_DIR"
    latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")
    cd ..
else
    latest_tag="v0.1.0"
fi

echo "  Versión detectada: $latest_tag"
tarball_url="https://github.com/$REPO/archive/refs/tags/$latest_tag.tar.gz"

# Calcular SHA256
echo "  Descargando $tarball_url ..."
sha256=$(curl -sL "$tarball_url" | shasum -a 256 | cut -d' ' -f1)
echo "  SHA256: $sha256"

# Actualizar PKGBUILD
echo "→ Actualizando $PKGBUILD ..."
version="${latest_tag#v}"
sed -i.bak \
    -e "s|^pkgver=.*|pkgver=$version|" \
    -e "s|sha256sums=('.*')|sha256sums=('$sha256')|" \
    "$PKGBUILD"

rm -f "$PKGBUILD.bak"

echo "✓ PKGBUILD actualizado"
echo ""
echo "Para verificar:"
echo "  cd $AUR_DIR"
echo "  makepkg -si"
echo ""
echo "Para subir a AUR:"
echo "  cd $AUR_DIR"
echo "  git add PKGBUILD .SRCINFO"
echo "  git commit -m 'Update to $latest_tag'"
echo "  git push"
