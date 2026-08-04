# Git Hero 🚀

> **[English](#english)** · **[Español](#español)**

A fast and visual Terminal UI (TUI) for Git, written in **Rust** with [Ratatui](https://ratatui.rs/).  
Run it with `gith`.

Inspired by tools like `lazygit` or `gitui`, but focused on being **simple, fast, and read-only** to visualize your repository state and execute common Git actions.

---

## English

A fast and visual Terminal UI (TUI) application for managing Git, written in **Rust** with [Ratatui](https://ratatui.rs/).

Inspired by tools like `lazygit` or `gitui`, but focused on being **simple, fast, and read-only** to visualize your repository state and execute common Git actions.

---

## ✨ Features

### Visualization
- **Integrated header** with ASCII logo, version badge, branch, behind/ahead indicators, and working directory
- **Files panel** with change indicators (modified, added, deleted, untracked)
- **Side-by-side diff** between current state and HEAD to see changes at a glance
- **Commit history** with expandable details
- **10 customizable themes** (Tokyo Night, Gruvbox Dark, Dracula, Nord, etc.)
- **Auto-update check** on startup via GitHub API

### Git Actions
- Stage/unstage individual files or all at once
- Create commits with message
- Undo last commit (with safety validation)
- Push, pull, fetch
- Create and switch between branches
- Stash and stash pop
- Configure remote
- Remove repository (with double confirmation)
- Copy diff to clipboard

### Usage Modes
- **TUI Mode** (default): Interactive visual interface
- **CLI Mode** (`-cli` or `-c`): Non-interactive flow for scripting

### Platform Support
- **Linux**: x86_64, aarch64
- **macOS**: Intel, Apple Silicon
- **Windows**: x86_64 (MSVC)

---

## 📦 Installation

### Quick Install (recommended)

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.ps1 | iex
```

**Linux / macOS / WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.sh | sh
```

**Homebrew (macOS / Linux):**
```bash
brew tap MarlonRX/git-hero
brew install gith
```

**Cargo (any platform):**
```bash
cargo install gith
```

### Download Prebuilt Binary

Download the latest binary for your platform from [GitHub Releases](https://github.com/MarlonRX/git-hero/releases/latest):

| Platform | File |
|----------|------|
| Linux x86_64 | `gith-*-linux-x86_64.tar.gz` |
| Linux aarch64 | `gith-*-linux-aarch64.tar.gz` |
| macOS Intel | `gith-*-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `gith-*-macos-aarch64.tar.gz` |
| Windows x86_64 | `gith-*-windows-x86_64.zip` |

### Build from Source

```bash
git clone https://github.com/MarlonRX/git-hero.git
cd gith
cargo build --release
```

The binary will be at `target/release/gith`. Move it to a directory in your `$PATH`.

---

## 🚀 Usage

### TUI Mode (interactive)
```bash
gith
# or after building from source:
./target/release/gith
```

### CLI Mode (non-interactive)
```bash
gith -cli
```

### Debug Mode
Generates detailed logs in `/tmp/git-hero-debug.log`:
```bash
gith --debug
tail -f /tmp/git-hero-debug.log
```

---

## ⌨️ Keyboard Shortcuts

### Navigation
| Key | Action |
|-------|--------|
| `Tab` | Switch focus between panels (files → diff → commits) |
| `↑/↓` or `k/j` | Move selection up/down |
| `Space` | Stage/unstage selected file |
| `Enter` | View commit detail (commits panel) |

### Git Actions
| Key | Action |
|-------|--------|
| `a` | Stage all files |
| `u` | Unstage all files |
| `c` | Create commit (opens input) |
| `r` | Undo last commit |
| `p` | Push |
| `f` | Fetch |
| `l` | Pull |
| `s` | Stash |
| `d` | Stash pop |
| `b` | List branches |
| `n` | Create new branch |
| `o` | Configure remote |
| `t` | Change theme |
| `y` | Copy diff to clipboard |

### Other
| Key | Action |
|-------|--------|
| `?` or `h` | Show help |
| `q` | Quit |
| `/` | Open command bar |
| `Ctrl+C` | Quit |

### Mouse
- **Click** on any panel → switch focus
- **Mouse wheel** on a panel → contextual scroll
- **Wheel on diff** → scroll diff
- **Wheel on commits** → scroll commit list

---

## 📂 Project Structure

```text
gith/
├── Cargo.toml                  # Dependencies and metadata
├── README.md                   # This file
├── build.rs                    # Build script (embeds git hash)
├── scripts/
│   ├── install.sh              # Unix installer (curl | sh)
│   ├── install.ps1             # Windows installer (PowerShell)
│   ├── deploy.sh               # Full deployment pipeline
│   ├── release.sh              # Cross-platform release builder
│   └── build-release.sh        # Local release build
├── .github/
│   └── workflows/
│       └── release.yml         # Automated CI/CD on tag push
└── src/
    ├── main.rs                 # Main entry and CLI args
    ├── config.rs               # Load/save user configuration
    ├── theme.rs                # 10 color themes
    ├── i18n.rs                 # English/Spanish translations
    ├── git.rs                  # Wrapper around system git commands
    ├── git_error.rs            # Git error types
    ├── cli.rs                  # CLI mode (non-interactive)
    ├── log.rs                  # Debug logging
    ├── version.rs              # Version info from Cargo.toml
    └── ui/
        ├── mod.rs              # UI module hub + event loop
        ├── modals.rs           # Modals (setup, theme, help, docs)
        ├── state/
        │   ├── mod.rs          # AppState, GitFile, GitCommit
        │   ├── command.rs      # Command parser and dispatch
        │   ├── commands.rs     # Command execution
        │   ├── icons.rs        # Nerd Font / ASCII icon tables
        │   └── suggestions.rs  # Command autocomplete
        ├── rendering/
        │   ├── mod.rs          # draw_ui(), header, footer
        │   ├── components.rs   # Layout, borders, diff renderer
        │   └── panels.rs       # Dashboard, files, commits panels
        └── events/
            ├── mod.rs          # Event routing
            ├── keyboard.rs     # Keyboard handlers
            └── mouse.rs        # Mouse handlers
```

---

## ⚙️ Configuration

The configuration file is saved at:
- **Linux**: `~/.config/git-hero/config.json`
- **macOS**: `~/Library/Application Support/git-hero/config.json`
- **Windows**: `%LOCALAPPDATA%\git-hero\config.json`

On first launch, a configuration wizard runs where you can choose:
1. Language (English / Español)
2. Use Nerd Font for icons
3. Theme

---

## 🎨 Included Themes

- Tokyo Night
- Gruvbox Dark
- Gruvbox Light
- Dracula
- Nord
- Solarized Dark
- Solarized Light
- One Dark
- Monokai
- Catppuccin

Switch themes with the `t` key.

---

## 🔄 Auto-Update

Git Hero checks for new versions on startup:
1. First tries `git ls-remote` (fast when git is in PATH)
2. Falls back to GitHub API via HTTP (works without git in PATH)
3. Shows a modal if a newer version is available
4. Opens the releases page in your default browser

---

## 🔧 Dependencies

| Crate | Version | Use |
|-------|---------|-----|
| `ratatui` | 0.30.1 | TUI framework |
| `crossterm` | 0.29.0 | Terminal backend and events |
| `dirs` | 6.0.0 | System home/config paths |
| `serde` | 1.0.228 | Configuration serialization |
| `serde_json` | 1.0.150 | JSON format for config |
| `phf` | 0.11 | Static icon maps |

---

## 📝 License

MIT

---

## Español

Aplicación de terminal (TUI) rápida y visual para gestionar Git, escrita en **Rust** con [Ratatui](https://ratatui.rs/).

Inspirada en herramientas como `lazygit` o `gitui`, pero enfocada en ser **simple, rápida y de solo lectura** para visualizar el estado de tu repositorio y ejecutar acciones comunes de Git.

---

## ✨ Características

### Visualización
- **Header integrado** con logo ASCII, badge de versión, rama, indicadores behind/ahead y directorio de trabajo
- **Panel de archivos** con indicadores de cambios (modificados, agregados, eliminados, sin trackear)
- **Diff side-by-side** entre el estado actual y HEAD para ver los cambios de un vistazo
- **Historial de commits** con detalles expandibles
- **10 temas** personalizables (Tokyo Night, Gruvbox Dark, Dracula, Nord, etc.)
- **Auto-actualización** al iniciar vía GitHub API

### Acciones de Git
- Stage/unstage de archivos individuales o todos a la vez
- Crear commits con mensaje
- Deshacer el último commit (con validación de seguridad)
- Push, pull, fetch
- Crear y cambiar entre ramas
- Stash y stash pop
- Configurar remote
- Eliminar el repositorio (con doble confirmación)
- Copiar diff al portapapeles

### Modos de uso
- **Modo TUI** (por defecto): Interfaz visual interactiva
- **Modo CLI** (`-cli` o `-c`): Flujo no interactivo para scripting

### Soporte de plataformas
- **Linux**: x86_64, aarch64
- **macOS**: Intel, Apple Silicon
- **Windows**: x86_64 (MSVC)

---

## 📦 Instalación

### Instalación rápida (recomendada)

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.ps1 | iex
```

**Linux / macOS / WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.sh | sh
```

**Homebrew (macOS / Linux):**
```bash
brew tap MarlonRX/git-hero
brew install gith
```

**Cargo (cualquier plataforma):**
```bash
cargo install gith
```

### Descargar binario precompilado

Descarga el último binario para tu plataforma desde [GitHub Releases](https://github.com/MarlonRX/git-hero/releases/latest):

| Plataforma | Archivo |
|------------|---------|
| Linux x86_64 | `gith-*-linux-x86_64.tar.gz` |
| Linux aarch64 | `gith-*-linux-aarch64.tar.gz` |
| macOS Intel | `gith-*-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `gith-*-macos-aarch64.tar.gz` |
| Windows x86_64 | `gith-*-windows-x86_64.zip` |

### Compilar desde fuente

```bash
git clone https://github.com/MarlonRX/git-hero.git
cd gith
cargo build --release
```

El binario estará en `target/release/gith`. Muévelo a un directorio en tu `$PATH`.

---

## 🚀 Uso

### Modo TUI (interactivo)
```bash
gith
# o después de compilar:
./target/release/gith
```

### Modo CLI (no interactivo)
```bash
gith -cli
```

### Modo Debug
Genera logs detallados en `/tmp/git-hero-debug.log`:
```bash
gith --debug
tail -f /tmp/git-hero-debug.log
```

---

## ⌨️ Atajos de Teclado

### Navegación
| Tecla | Acción |
|-------|--------|
| `Tab` | Cambia foco entre panels (files → diff → commits) |
| `↑/↓` o `k/j` | Mover selección arriba/abajo |
| `Espacio` | Stage/unstage archivo seleccionado |
| `Enter` | Ver detalle del commit (panel commits) |

### Acciones de Git
| Tecla | Acción |
|-------|--------|
| `a` | Stage todos los archivos |
| `u` | Unstage todos los archivos |
| `c` | Crear commit (abre input) |
| `r` | Deshacer último commit |
| `p` | Push |
| `f` | Fetch |
| `l` | Pull |
| `s` | Stash |
| `d` | Stash pop |
| `b` | Listar ramas |
| `n` | Crear nueva rama |
| `o` | Configurar remote |
| `t` | Cambiar tema |
| `y` | Copiar diff al portapapeles |

### Otros
| Tecla | Acción |
|-------|--------|
| `?` o `h` | Mostrar ayuda |
| `q` | Salir |
| `/` | Abrir barra de comandos |
| `Ctrl+C` | Salir |

### Mouse
- **Click** en cualquier panel → cambia el foco
- **Rueda del mouse** sobre un panel → scroll contextual
- **Rueda en el diff** → scroll del diff
- **Rueda en commits** → scroll de la lista de commits

---

## 📂 Estructura del Proyecto

```text
gith/
├── Cargo.toml                  # Dependencias y metadata
├── README.md                   # Este archivo
├── build.rs                    # Build script (embebe hash de git)
├── scripts/
│   ├── install.sh              # Instalador Unix (curl | sh)
│   ├── install.ps1             # Instalador Windows (PowerShell)
│   ├── deploy.sh               # Pipeline de deployment completo
│   ├── release.sh              # Builder de release multi-plataforma
│   └── build-release.sh        # Build de release local
├── .github/
│   └── workflows/
│       └── release.yml         # CI/CD automático al push de tag
└── src/
    ├── main.rs                 # Entrada principal y CLI args
    ├── config.rs               # Carga/guarda configuración del usuario
    ├── theme.rs                # 10 temas con colores
    ├── i18n.rs                 # Traducciones inglés/español
    ├── git.rs                  # Wrapper sobre comandos git del sistema
    ├── git_error.rs            # Tipos de error de git
    ├── cli.rs                  # Modo CLI (no interactivo)
    ├── log.rs                  # Logging de debug
    ├── version.rs              # Info de versión desde Cargo.toml
    └── ui/
        ├── mod.rs              # Hub del módulo UI + event loop
        ├── modals.rs           # Modales (setup, theme, help, docs)
        ├── state/
        │   ├── mod.rs          # AppState, GitFile, GitCommit
        │   ├── command.rs      # Parser y dispatch de comandos
        │   ├── commands.rs     # Ejecución de comandos
        │   ├── icons.rs        # Tablas de iconos Nerd Font / ASCII
        │   └── suggestions.rs  # Autocompletado de comandos
        ├── rendering/
        │   ├── mod.rs          # draw_ui(), header, footer
        │   ├── components.rs   # Layout, bordes, renderizado de diff
        │   └── panels.rs       # Dashboard, panels de archivos/commits
        └── events/
            ├── mod.rs          # Enrutamiento de eventos
            ├── keyboard.rs     # Handlers de teclado
            └── mouse.rs        # Handlers de mouse
```

---

## ⚙️ Configuración

El archivo de configuración se guarda en:
- **Linux**: `~/.config/git-hero/config.json`
- **macOS**: `~/Library/Application Support/git-hero/config.json`
- **Windows**: `%LOCALAPPDATA%\git-hero\config.json`

Al primer inicio se ejecuta un asistente de configuración donde puedes elegir:
1. Idioma (English / Español)
2. Usar Nerd Font para iconos
3. Tema

---

## 🎨 Temas Incluidos

- Tokyo Night
- Gruvbox Dark
- Gruvbox Light
- Dracula
- Nord
- Solarized Dark
- Solarized Light
- One Dark
- Monokai
- Catppuccin

Cambia de tema con la tecla `t`.

---

## 🔄 Auto-actualización

Git Hero verifica nuevas versiones al iniciar:
1. Primero intenta `git ls-remote` (rápido cuando git está en PATH)
2. Usa la GitHub API vía HTTP como fallback (funciona sin git en PATH)
3. Muestra un modal si hay una versión más reciente disponible
4. Abre la página de releases en tu navegador predeterminado

---

## 🔧 Dependencias

| Crate | Versión | Uso |
|-------|---------|-----|
| `ratatui` | 0.30.1 | Framework TUI |
| `crossterm` | 0.29.0 | Backend de terminal y eventos |
| `dirs` | 6.0.0 | Rutas del sistema home/config |
| `serde` | 1.0.228 | Serialización de configuración |
| `serde_json` | 1.0.150 | Formato JSON para config |
| `phf` | 0.11 | Mapas estáticos de iconos |

---

## 📝 Licencia

MIT
