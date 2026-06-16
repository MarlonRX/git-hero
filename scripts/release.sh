#!/bin/bash
# ============================================================================
# Git Hero - Release Script
# Compila binarios para múltiples plataformas y crea releases
# ============================================================================
set -euo pipefail

# ── Configuración ─────────────────────────────────────────────────────────
APP_NAME="git-hero"
VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"
RELEASE_DIR="target/release-artifacts"
BINARY_NAME="$APP_NAME"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         git-hero Release Builder v${VERSION}                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# ── Verificar versión ────────────────────────────────────────────────────
if [ -z "$VERSION" ]; then
    echo "✗ Error: no se pudo determinar la versión"
    echo "  Uso: $0 [version]"
    echo "  Ejemplo: $0 0.1.0"
    exit 1
fi

# ── Limpiar builds anteriores ────────────────────────────────────────────
echo "→ Limpiando builds anteriores..."
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"
cargo clean 2>/dev/null || true

# ── Targets a compilar ───────────────────────────────────────────────────
TARGETS=(
    "x86_64-unknown-linux-gnu:linux:x86_64:gnu"
    "x86_64-unknown-linux-musl:linux:x86_64:musl"
    "aarch64-unknown-linux-gnu:linux:aarch64:gnu"
    "x86_64-apple-darwin:macos:x86_64:darwin"
    "aarch64-apple-darwin:macos:aarch64:darwin"
    "x86_64-pc-windows-msvc:windows:x86_64:msvc.exe"
)

# ── Compilar para cada target ────────────────────────────────────────────
for target_info in "${TARGETS[@]}"; do
    IFS=':' read -ra parts <<< "$target_info"
    target="${parts[0]}"
    os="${parts[1]}"
    arch="${parts[2]}"
    suffix="${parts[3]}"

    echo ""
    echo "→ Compilando para $os ($arch) [$target]..."

    # Verificar si el target está instalado
    if ! rustup target list --installed | grep -q "$target"; then
        echo "  ⚠ Instalando target $target..."
        rustup target add "$target" 2>/dev/null || {
            echo "  ✗ No se pudo instalar target $target, saltando..."
            continue
        }
    fi

    # Compilar
    if cargo build --release --target "$target" 2>/dev/null; then
        # Determinar extensión del binario
        if [[ "$target" == *"windows"* ]]; then
            binary_ext=".exe"
        else
            binary_ext=""
        fi

        binary_path="target/$target/release/${BINARY_NAME}${binary_ext}"
        if [ -f "$binary_path" ]; then
            # Crear directorio de release
            artifact_dir="$RELEASE_DIR/${APP_NAME}-${VERSION}-${os}-${arch}"
            mkdir -p "$artifact_dir"

            # Copiar binario
            cp "$binary_path" "$artifact_dir/${BINARY_NAME}${binary_ext}"
            chmod +x "$artifact_dir/${BINARY_NAME}${binary_ext}"

            # Copiar documentación
            cp README.md "$artifact_dir/"
            cp LICENSE "$artifact_dir/" 2>/dev/null || true

            # Crear archivo comprimido
            cd "$RELEASE_DIR"
            if [[ "$target" == *"windows"* ]]; then
                zip -qr "${artifact_dir}.zip" "${artifact_dir##*/}"
                echo "  ✓ Creado: ${artifact_dir##*/}.zip"
            else
                tar -czf "${artifact_dir}.tar.gz" "${artifact_dir##*/}"
                echo "  ✓ Creado: ${artifact_dir##*/}.tar.gz"
            fi
            cd - > /dev/null
        else
            echo "  ✗ Binario no encontrado: $binary_path"
        fi
    else
        echo "  ✗ Error compilando para $target"
    fi
done

# ── Generar checksums ────────────────────────────────────────────────────
echo ""
echo "→ Generando checksums..."
cd "$RELEASE_DIR"
for file in *.tar.gz *.zip; do
    if [ -f "$file" ]; then
        shasum -a 256 "$file" >> checksums.txt
    fi
done
cat checksums.txt
cd - > /dev/null

# ── Resumen ──────────────────────────────────────────────────────────────
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    Build completo                           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Artefactos generados en: $RELEASE_DIR/"
ls -lh "$RELEASE_DIR"/ | grep -E '\.(tar\.gz|zip|txt)$' || true
echo ""
echo "Próximos pasos:"
echo "  1. Crear tag: git tag -a v$VERSION -m 'Release v$VERSION'"
echo "  2. Push tag: git push origin v$VERSION"
echo "  3. Crear release en GitHub: gh release create v$VERSION $RELEASE_DIR/*"
echo "  4. Publicar en crates.io: cargo publish"
echo "  5. Actualizar Homebrew tap con el SHA256"
