#!/bin/bash
# ============================================================================
# Update Homebrew Formula with SHA256
# Calcula automáticamente el SHA256 del tarball y actualiza la fórmula
# ============================================================================
set -euo pipefail

REPO="MarlonRX/git-hero"
TAP_DIR="homebrew-tap"
FORMULA="$TAP_DIR/git-hero.rb"

echo "→ Calculando SHA256 del tarball de la release..."

# Detectar última versión
if [ -d "$TAP_DIR/.git" ]; then
    cd "$TAP_DIR"
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

# Actualizar fórmula
echo "→ Actualizando $FORMULA ..."
version="${latest_tag#v}"
sed -i.bak \
    -e "s|url \".*\"|url \"https://github.com/$REPO/archive/refs/tags/$latest_tag.tar.gz\"|" \
    -e "s|sha256 \".*\"|sha256 \"$sha256\"|" \
    -e "s|v[0-9]*\.[0-9]*\.[0-9]*\.tar\.gz|v$version.tar.gz|g" \
    "$FORMULA"

rm -f "$FORMULA.bak"

echo "✓ Fórmula actualizada"
echo ""
echo "Próximos pasos:"
echo "  1. cd $TAP_DIR"
echo "  2. git add git-hero.rb"
echo "  3. git commit -m 'Update git-hero to $latest_tag'"
echo "  4. git push"
