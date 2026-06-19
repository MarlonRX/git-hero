# Git Hero 🚀

> **[English](#english)** · **[Español](#español)**

A fast and visual Terminal UI (TUI) for Git, written in **Rust** with [Ratatui](https://ratatui.rs/).  
Run it with `gith`.

Inspired by tools like `lazygit` or `gitui`, but focused on being **simple, fast, and read-only** to visualize your repository state and execute common Git actions.

---

## English

Una aplicación de terminal (TUI) rápida y visual para gestionar Git, escrita en **Rust** con [Ratatui](https://ratatui.rs/).

Inspirada en herramientas como `lazygit` o `gitui`, pero enfocada en ser **simple, rápida y de solo lectura** para visualizar el estado de tu repositorio y ejecutar acciones comunes de Git.

---

## ✨ Features

### Visualization
- **Visual dashboard** with repository status (branch, remote, ahead/behind)
- **Files panel** with change indicators (modified, added, deleted, untracked)
- **Side-by-side diff** between current state and HEAD to see changes at a glance
- **Commit history** with expandable details
- **10 customizable themes** (Tokyo Night, Gruvbox Dark, Dracula, Nord, etc.)

### Git Actions
- Stage/unstage individual files or all at once
- Create commits with message
- Undo last commit (with safety validation)
- Push, pull, fetch
- Create and switch between branches
- Stash and stash pop
- Configure remote
- Remove repository (with double confirmation)

### Usage Modes
- **TUI Mode** (default): Interactive visual interface
- **CLI Mode** (`-cli` or `-c`): Non-interactive flow for scripting

---

## 📦 Installation

### Quick Install (recommended)

**Linux / macOS / WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/MarlonRX/git-hero/main/scripts/install.sh | bash
```

**Homebrew (macOS / Linux):**
```bash
brew tap MarlonRX/git-hero
brew install gith
```

**Cargo (any platform, including Windows):**
```bash
cargo install gith
```

**Windows (Scoop):**
```powershell
scoop bucket add MarlonRX https://github.com/MarlonRX/git-hero
scoop install gith
```

**Windows (manual):**
1. Install Rust from [rustup.rs](https://rustup.rs/)
2. Open a terminal (PowerShell / cmd / Git Bash)
3. Run:
   ```powershell
   cargo install gith
   ```

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
├── Cargo.toml              # Dependencies (ratatui, crossterm, dirs, serde)
├── README.md               # This file
├── .gitignore              # Ignored files
└── src/
    ├── main.rs             # Main entry and CLI args
    ├── config.rs           # Load/save user configuration
    ├── theme.rs            # 10 color themes
    ├── i18n.rs             # English/Spanish translations
    ├── git.rs              # Wrapper around system git commands
    ├── cli.rs              # CLI mode (non-interactive)
    └── ui/
        ├── mod.rs          # UI module hub + event loop
        ├── state.rs        # AppState, GitFile, GitCommit
        ├── rendering.rs    # draw_ui(), panel drawing, diff renderer
        ├── modals.rs        # Modals (setup, theme, help, docs)
        └── events.rs        # Keyboard and mouse handlers
```

---

## ⚙️ Configuration

The configuration file is saved at:
- **Linux**: `~/.config/git-hero/config.json`
- **macOS**: `~/Library/Application Support/git-hero/config.json`

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

## 🔧 Dependencies

| Crate | Version | Use |
|-------|---------|-----|
| `ratatui` | 0.30.1 | TUI framework |
| `crossterm` | 0.29.0 | Terminal backend and events |
| `dirs` | 6.0.0 | System home/config paths |
| `serde` | 1.0.228 | Configuration serialization |
| `serde_json` | 1.0.150 | JSON format for config |

---

## 📝 License

MIT

---

## Español

A fast and visual Terminal UI (TUI) application for managing Git, written in **Rust** with [Ratatui](https://ratatui.rs/).

Inspired by tools like `lazygit` or `gitui`, but focused on being **simple, fast, and read-only** to visualize your repository state and execute common Git actions.

---

## ✨ Características

### Visualización
- **Dashboard visual** con estado del repositorio (rama, remoto, ahead/behind)
- **Panel de archivos** con indicadores de cambios (modificados, agregados, eliminados, sin trackear)
- **Diff side-by-side** entre el estado actual y HEAD para ver los cambios de un vistazo
- **Historial de commits** con detalles expandibles
- **10 temas** personalizables (Tokyo Night, Gruvbox Dark, Dracula, Nord, etc.)

### Acciones de Git
- Stage/unstage de archivos individuales o todos a la vez
- Crear commits con mensaje
- Deshacer el último commit (con validación de seguridad)
- Push, pull, fetch
- Crear y cambiar entre ramas
- Stash y stash pop
- Configurar remote
- Eliminar el repositorio (con doble confirmación)

### Modos de uso
- **Modo TUI** (por defecto): Interfaz visual interactiva
- **Modo CLI** (`-cli` o `-c`): Flujo no interactivo para scripting

---

## 📦 Instalación

### Prerrequisitos
- Rust 1.75+ ([instalar desde rustup.rs](https://rustup.rs/))
- Git instalado y disponible en `$PATH`

### Compilar
```bash
cargo build --release
```

El binario estará en `target/release/gith`. Puedes moverlo a `/usr/local/bin/` para tenerlo disponible globalmente.

---

## 🚀 Uso

### Modo TUI (interactivo)
```bash
cargo run
# o después de compilar:
./target/release/gith
```

### Modo CLI (no interactivo)
```bash
cargo run -- -cli
```

### Modo Debug
Genera logs detallados en `/tmp/git-hero-debug.log`:
```bash
cargo run -- --debug
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
├── Cargo.toml              # Dependencies (ratatui, crossterm, dirs, serde)
├── README.md               # This file
├── .gitignore              # Ignored files
└── src/
    ├── main.rs             # Entrada principal y CLI args
    ├── config.rs           # Carga/guarda configuración del usuario
    ├── theme.rs            # 10 temas con colores
    ├── i18n.rs             # Traducciones inglés/español
    ├── git.rs              # Wrapper sobre comandos git del sistema
    ├── cli.rs              # Modo CLI (no interactivo)
    └── ui/
        ├── mod.rs          # Hub del módulo UI + event loop
        ├── state.rs        # AppState, GitFile, GitCommit
        ├── rendering.rs    # draw_ui(), panel drawing, diff renderer
        ├── modals.rs       # Modales (setup, theme, help, docs)
        └── events.rs       # Handlers de teclado y mouse
```

---

## ⚙️ Configuración

El archivo de configuración se guarda en:
- **Linux**: `~/.config/git-hero/config.json`
- **macOS**: `~/Library/Application Support/git-hero/config.json`

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

## 🔧 Dependencias

| Crate | Versión | Uso |
|-------|---------|-----|
| `ratatui` | 0.30.1 | Framework TUI |
| `crossterm` | 0.29.0 | Backend de terminal y eventos |
| `dirs` | 6.0.0 | Rutas del sistema home/config |
| `serde` | 1.0.228 | Serialización de configuración |
| `serde_json` | 1.0.150 | Formato JSON para config |

---

## 📝 Licencia

MIT

