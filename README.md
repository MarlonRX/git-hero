# Git Hero 🚀

> **[English](#english)** · **[Español](#español)**

**A simple repository *manager* for the terminal — not another Git client.**
Written in **Rust** with [Ratatui](https://ratatui.rs/). Run it with `gith`.

Git Hero's niche: you have dozens of repositories, and you mostly want to know
**which ones have junk to commit, which are behind their remote, what branch is
checked out, and when you last touched each one** — then jump in, do the chore,
and move on. That's what `gith` is for.

## The five everyday actions

Everything in Git Hero serves the repository-manager story:

| # | Action | How | Safety |
|---|--------|-----|--------|
| 1 | **See state** | files + side-by-side diff + history + `↑ahead/↓behind` badges | read-only, auto-refresh every 2 s |
| 2 | **Commit** | `c` / `/commit` — stage & commit, multi-line message | confirmation-free but undoable (`r`) |
| 3 | **Pull** | `l` / `/pull` | explicit y/N modal |
| 4 | **Push** | `p` / `/push` | explicit y/N modal |
| 5 | **Switch branch** | `/switch <name>` / `/branch` | instant, errors reported |

Plus the manager glue: `g` or `/repos` opens a **Repository Browser** — a
dropdown in the sidebar (where the shortcuts block used to live) that
**recursively** scans your projects folder and lists every repo with its
branch, `↑ahead/↓behind`, dirty-file count and last-activity age. Just **type
to filter** (no git calls while typing), `Enter` or click to jump into a repo,
`Ctrl+R` to re-scan, `Esc` to close. **Open `gith` in a plain folder and this
browser is what you get first** — pick a repo from the list or use the
init/`/cd` options beside it. The repository you are currently inside is
tagged `● here`, and your location is a bold badge in the header.

```text
┌ FILES (3) ─────────┬────────────────────────────────────────┐
│  ✓ src/i18n.rs     │  DIFF                                  │
│  ▶ src/cli.rs      │  ...                                   │
├────────────────────┤                                        │
│ ◆ Repositories      │                                        │
│  2 of 4 dirty      │                                        │
│  ⌕ ser█  2/4       │                                        │
│  ▶ work/api-serv…  │                                        │
│    web/api-gw      │                                        │
│  type=filter Enter │                                        │
│  =open Ctrl+R=scan │                                        │
└────────────────────┴────────────────────────────────────────┘
```

**What Git Hero is *not*:** an interactive-rebase tool, a blame viewer or a
submodule IDE. For deep single-repo surgery use `lazygit`/`gitui` — we don't
compete there and we won't grow features that break the "simple manager" story.

---

## English

A fast Terminal UI for **managing many Git repositories**: overview scan,
state at a glance, quick repo-to-repo jumps, and the five actions above done
comfortably. Bilingual (EN/ES), 10 themes, Nerd Font support, and a
non-interactive `-cli` mode for scripting the same commit→pull→push flow.

---

## ✨ Features

### Repository management (the niche)
- **Repository Browser** (`g` or `/repos`): type-ahead dropdown in the sidebar — recursively scans your projects folder (depth-bounded, vendor dirs skipped) and lists branch, ahead/behind, dirty count and last-activity age per repo, sorted by recency. Typing filters with zero git calls; Enter or click jumps in; `Ctrl+R` re-scans
- **Status at a glance**: header with branch, `↑ahead`/`↓behind` badges and working directory
- **Auto-refresh** of the current repo every 2 seconds (zero git calls when nothing changed)

### Visualization
- **Files panel** with change indicators (modified, added, deleted, untracked)
- **Side-by-side diff** between current state and HEAD to see changes at a glance
- **Commit history** with expandable details and pushed/unpushed markers
- **10 customizable themes** (Tokyo Night, Gruvbox Dark, Dracula, Nord, etc.)
- **Bilingual UI** (English / Spanish), switch live with `/language en|es`
- **Auto-update check** on startup via GitHub API

### Git Actions (the five + helpers)
- Stage/unstage individual files or all at once
- Create commits with a multi-line message editor
- Push, pull, fetch (push/pull behind confirmation modals)
- Create and switch between branches
- Undo last commit (with safety validation)
- Stash and stash pop
- Configure remote
- Remove repository `.git` — always behind a confirmation modal
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
| `g` | Repository browser (type to filter, Enter to jump) |
| `y` | Copy diff to clipboard |

### Other
| Key | Action |
|-------|--------|
| `?` or `h` | Show help |
| `q` | Quit |
| `/` | Open command bar (works from anywhere, incl. the repo browser) |
| `Ctrl+C` | Quit |
| `PgUp/PgDn` / wheel | Page through files, commits and the browser |

> The footer always shows a `[key] action` strip for the context you are in
> (repo, browser, no-repo, typing a command), so the shortcuts are visible
> without opening `/help`.

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
        │   ├── icons.rs        # Nerd Font / ASCII icon tables (phf)
        │   ├── repos.rs        # Multi-repo scan (Repository Overview)
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
- **Windows**: `%APPDATA%\git-hero\config.json`

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
| `phf` | 0.11 | Compile-time static maps (icons, i18n dictionaries) |

---

## 📝 License

MIT

---

## Español

**Git Hero es un *gestor* de repositorios para la terminal — no un cliente Git más.**
Escrito en **Rust** con [Ratatui](https://ratatui.rs/). Se ejecuta con `gith`.

El nicho de Git Hero: tenés decenas de repos y lo que querés saber de un vistazo
es **cuáles tienen basura sin commitear, cuáles están detrás de su remoto, qué
rama está activa y cuándo los tocaste por última vez** — entrar, resolver la
tarea y seguir. Las cinco acciones del día a día:

| # | Acción | Cómo | Seguridad |
|---|--------|------|-----------|
| 1 | **Ver estado** | archivos + diff side-by-side + historial + badges `↑adelante/↓detrás` | solo lectura, auto-refresh cada 2 s |
| 2 | **Commit** | `c` / `/commit` — stagea y commitea, mensaje multilínea | deshacible (`r`) |
| 3 | **Pull** | `l` / `/pull` | modal de confirmación y/N explícito |
| 4 | **Push** | `p` / `/push` | modal de confirmación y/N explícito |
| 5 | **Cambiar de rama** | `/switch <nombre>` / `/branch` | instantáneo, errores reportados |

Y la cola de gestor: `g` o `/repos` abre el **Navegador de Repositorios** —
un desplegable en la barra lateral (donde antes vivían los shortcuts) que
recorre **recursivamente** tu carpeta de proyectos y lista cada repo con su
rama, `↑adelante/↓detrás`, cantidad de archivos sucios y antigüedad del último
commit. **Escribí para filtrar** (sin llamadas a git mientras tipeás), Enter o
click para entrar, `Ctrl+R` reescanea, `Esc` cierra. **Si abrís `gith` en una
carpeta sin repo, esto es lo primero que vas a ver** — elegí un repo de la
lista o usá las opciones de init/`/cd` al lado. El repo donde estás ahora
aparece marcado `● actual` y tu ubicación va en un badge grande y destacado
en el header.

**Lo que Git Hero *no* es:** una herramienta de rebase interactivo, visor de
blame ni IDE de submódulos. Para cirugía profunda de un repo usá `lazygit`/`gitui`
— ahí no competimos.

---

## ✨ Características

### Gestión de repositorios (el nicho)
- **Navegador de Repositorios** (`g` o `/repos`): desplegable con filtrado al teclear en la barra lateral — escanea recursivamente tu carpeta de proyectos (profundidad limitada, salta carpetas de vendor/build) y lista rama, adelante/detrás, archivos sucios y antigüedad por repo, ordenados por actividad. Escribir filtra sin gastar llamadas a git; Enter o click entra al repo; `Ctrl+R` reescanea
- **Estado de un vistazo**: rama, badges `↑adelante`/`↓detrás` y directorio actual
- **Auto-refresco** del repo actual cada 2 segundos (cero llamadas a git si nada cambió)

### Visualización
- **Panel de archivos** con indicadores de cambios (modificados, agregados, eliminados, sin trackear)
- **Diff side-by-side** entre el estado actual y HEAD para ver los cambios de un vistazo
- **Historial de commits** con detalles expandibles y marcadores de push
- **10 temas** personalizables (Tokyo Night, Gruvbox Dark, Dracula, Nord, etc.)
- **Interfaz bilingüe** (English / Español), se cambia en vivo con `/language es`
- **Auto-actualización** al iniciar vía GitHub API

### Acciones de Git
- Stage/unstage de archivos individuales o todos a la vez
- Crear commits con mensaje
- Deshacer el último commit (con validación de seguridad)
- Push, pull, fetch
- Crear y cambiar entre ramas
- Stash y stash pop
- Configurar remote
- Eliminar el `.git` del repositorio — siempre detrás de un modal de confirmación
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
| `g` | Navegador de repos (escribir filtra, Enter entra) |
| `y` | Copiar diff al portapapeles |

### Otros
| Tecla | Acción |
|-------|--------|
| `?` o `h` | Mostrar ayuda |
| `q` | Salir |
| `/` | Abrir barra de comandos (funciona desde cualquier lado, incluido el navegador) |
| `Ctrl+C` | Salir |
| `PgUp/PgDn` / rueda | Paging de archivos, commits y navegador |

> El footer siempre muestra una tira `[tecla] acción` del contexto actual
> (repo, navegador, sin repo, escribiendo comando): los atajos están a la
> vista sin abrir `/help`.

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
        │   ├── icons.rs        # Tablas de iconos Nerd Font / ASCII (phf)
        │   ├── repos.rs        # Escaneo multi-repo (Resumen de Repositorios)
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
- **Windows**: `%APPDATA%\git-hero\config.json`

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
