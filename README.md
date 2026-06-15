# gacp-rx 🚀 (Git Add, Commit, Push - Redesigned)

Bienvenido a **gacp-rx**, la evolución y extensión de tu script interactivo `gacp` a una aplicación de interfaz de terminal (TUI) completa, rápida y optimizada.

El objetivo de este proyecto es transformar tu flujo de trabajo de Git actual (que realiza análisis remoto, adición, commit, pull y push) en una herramienta visual e interactiva en la terminal, inspirada en herramientas modernas como `lazygit` o `gitui`, pero optimizada y adaptada específicamente a tu flujo rápido.

---

## 🛠️ Opciones de Stack Tecnológico

Dado tu interés en lenguajes rápidos y optimizados como **Rust** y **Lua**, aquí tienes las dos opciones principales para construir `gacp-rx`, con sus ventajas y desventajas:

### Opción 1: Rust + Ratatui (Recomendada para herramienta independiente)
Rust es actualmente el estándar de la industria para construir CLI y TUI veloces, eficientes y seguras.

*   **Lenguaje:** Rust (compilado, sin recolector de basura, huella de memoria mínima).
*   **Framework TUI:** [Ratatui](https://ratatui.rs/) (el sucesor espiritual de `tui-rs`, muy activo, basado en ciclos de dibujado inmediato).
*   **Backend de Terminal:** `crossterm` (multiplataforma, maneja eventos de teclado/mouse y códigos ANSI).
*   **Integración con Git:**
    *   *Opción Ligera (Recomendada):* Ejecutar comandos de Git del sistema mediante `std::process::Command`. Esto asegura compatibilidad nativa con tus claves GPG (firma de commits), agentes SSH, credenciales y hooks locales de Git.
    *   *Opción Embebida:* `git2-rs` (bindings de `libgit2`). Más rápida para consultas complejas, pero más compleja de configurar con SSH/GPG.

#### Ventajas:
*   Genera un **único ejecutable binario** autocontenido, ultra veloz, que puedes mover a cualquier carpeta (`/usr/local/bin`, etc.) sin dependencias.
*   Tipado fuerte que previene errores en tiempo de ejecución.
*   Ecosistema de widgets TUI gigante (listas, tablas, inputs de texto, barras de progreso, etc.).

---

### Opción 2: Lua + Neovim Plugin (Recomendada si usas Neovim)
Lua es extremadamente rápido (gracias a LuaJIT) y ligero, pero su ecosistema de TUI independientes es más limitado. Sin embargo, brilla si lo construyes como una extensión de Neovim.

*   **Lenguaje:** Lua 5.1 / LuaJIT.
*   **Entorno:** Integrado dentro de Neovim usando librerías como `nui.nvim` o `plenary.nvim`.
*   **Integración Git:** Mediante llamadas asíncronas a comandos de Git.

#### Ventajas:
*   Si ya usas Neovim, la integración es 100% natural, compartiendo tus buffers y atajos.
*   Iteración y scripting sumamente rápidos.
*   Curva de aprendizaje muy baja.

#### Desventajas:
*   Para ejecutarlo como un comando independiente del sistema fuera de Neovim, requiere un intérprete de Lua instalado y la instalación de dependencias de terceros es más compleja de distribuir a otros usuarios.

---

## 🎯 Características Planeadas para `gacp-rx`

1.  **Dashboard Principal Dinámico:**
    *   Estado de la rama local vs. remota (commits *ahead* / *behind*) obtenido asíncronamente en segundo plano.
    *   Panel visual del estado de archivos (modificados, eliminados, no rastreados).
2.  **Gestión de staging Interactiva:**
    *   Permitir seleccionar archivos individuales con la tecla `Espacio` para agregarlos (stage) o quitarlos (unstage).
3.  **Editor de Mensajes de Commit Embebido:**
    *   Un modal o caja de texto interactiva para escribir el mensaje de commit directamente, con soporte para atajos de edición básicos.
4.  **Confirmaciones Inteligentes:**
    *   Pop-ups rápidos para aceptar `git pull` o `git push` con teclas dedicadas (ej. `y`/`n`).
5.  **Historial Visual (Log):**
    *   Un panel opcional para visualizar los últimos commits del repositorio.

---

## 📂 Estructura de Archivos del Proyecto (Propuesta para Rust)

Si decidimos ir por la opción de **Rust**, esta será la estructura base:

```text
gacp-rx/
├── Cargo.toml          # Configuración de dependencias (ratatui, crossterm, etc.)
├── README.md           # Este archivo
├── ROADMAP.md          # Plan de desarrollo paso a paso
└── src/
    ├── main.rs         # Entrada de la aplicación e inicialización de la terminal
    ├── app.rs          # Estado del TUI (pantallas, listas, inputs)
    ├── ui.rs           # Renderizado de componentes gráficos de Ratatui
    ├── event.rs        # Loop de eventos (teclado, ticks)
    └── git.rs          # Interfaz de comunicación con comandos Git
```

---

*Nota: He creado el archivo [ROADMAP.md](file:///Users/mramirez/Projects/gacp-rx/ROADMAP.md) en esta misma carpeta para detallar las fases del proyecto y guiarte en el desarrollo paso a paso.*

---

## 📜 Código de Referencia Original (Zsh Script)

Para facilitar la transición y el desarrollo paso a paso, aquí tienes el código original de tu función `gacp` en `.zshrc`:

```zsh
gacp() {
  local green='\033[0;32m'
  local blue='\033[0;34m'
  local yellow='\033[1;33m'
  local red='\033[0;31m'
  local cyan='\033[0;36m'
  local bold='\033[1m'
  local nc='\033[0m'

  local sep="${blue}════════════════════════════════════════${nc}"

  echo ""

  if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo "${red}  ✖  No estas dentro de un repositorio git.${nc}"
    return 1
  fi

  local branch=$(git symbolic-ref --short HEAD 2>/dev/null)
  local remote=$(git config branch.$branch.remote 2>/dev/null || echo "origin")

  echo "${sep}"
  echo "${bold}  GIT FLOW  ${nc}${cyan}${branch}${nc} ${blue}→${nc} ${cyan}${remote}${nc}"
  echo "${sep}"

  echo ""
  echo "${blue}  🔍 Verificando remoto...${nc}"
  git fetch $remote $branch 2>/dev/null

  local behind=$(git rev-list --count HEAD..$remote/$branch 2>/dev/null)
  local ahead=$(git rev-list --count $remote/$branch..HEAD 2>/dev/null)

  echo "${cyan}  ┌─ Remoto: ${remote}/${branch}${nc}"
  echo "${cyan}  ├─ Adelante: ${green}${ahead}${nc} commit(s) (por enviar)"
  echo "${cyan}  └─ Detras:   ${yellow}${behind}${nc} commit(s) (por recibir)"

  local did_commit=false
  local has_unpushed=false
  [ "$ahead" -gt 0 ] 2>/dev/null && has_unpushed=true

  # ── ADD + COMMIT (solo si hay cambios sin commitear) ──
  if [ -n "$(git status --porcelain)" ]; then
    echo ""
    echo "${blue}  📦 git add .${nc}"
    git add . || { echo "${red}  ✖  Error en git add.${nc}"; return 1; }

    if git diff --cached --quiet 2>/dev/null; then
      echo "${cyan}  ℹ  Sin cambios para commit (posibles ignorados por .gitignore).${nc}"
    else
      echo ""
      echo "${bold}  💬 Mensaje del commit:${nc}"
      echo -n "  ${blue}→${nc} "
      read commit_msg
      [ -z "$commit_msg" ] && { echo ""; echo "${red}  ✖  Mensaje vacio, cancelado.${nc}"; return 1; }

      echo ""
      echo "${blue}  📝 git commit -m \"${commit_msg}\"${nc}"
      git commit -m "$commit_msg" || { echo "${red}  ✖  Error al hacer commit.${nc}"; return 1; }
      echo "${green}  ✔  Commit creado${nc}"
      did_commit=true
      ahead=$(git rev-list --count $remote/$branch..HEAD 2>/dev/null)
      [ "$ahead" -gt 0 ] 2>/dev/null && has_unpushed=true
    fi
  else
    echo ""
    echo "${cyan}  ℹ  Sin cambios pendientes para commit.${nc}"
  fi

  # ── PULL ──
  if [ "$behind" -gt 0 ] 2>/dev/null; then
    echo ""
    echo "${yellow}  ⚠  El remoto tiene ${behind} commit(s) que no tienes.${nc}"
    echo -n "${bold}  ⬇  ¿Hacer pull? (s/N): ${nc}"
    read do_pull
    if [[ "$do_pull" =~ ^[sS]$ ]]; then
      echo ""
      echo "${blue}  ⬇  git pull ${remote} ${branch}${nc}"
      git pull $remote $branch || { echo "${red}  ✖  Error en pull.${nc}"; return 1; }
      echo "${green}  ✔  Pull completado${nc}"
      ahead=$(git rev-list --count $remote/$branch..HEAD 2>/dev/null)
      [ "$ahead" -gt 0 ] 2>/dev/null && has_unpushed=true
    fi
  elif ! $did_commit && ! $has_unpushed; then
    echo ""
    echo "${green}  ✔  Estas al dia con el remoto${nc}"
  fi

  # ── PUSH ──
  if $has_unpushed; then
    echo ""
    echo -n "${bold}  🚀 ¿Hacer push? (s/N): ${nc}"
    read do_push
    if [[ "$do_push" =~ ^[sS]$ ]]; then
      echo ""
      echo "${blue}  ⬆  git push ${remote} ${branch}${nc}"
      git push $remote $branch || { echo "${red}  ✖  Error en push.${nc}"; return 1; }
      echo "${green}  ✔  Push completado${nc}"
    else
      echo "${cyan}  📌 Commits locales guardados, sin push.${nc}"
    fi
  fi

  echo ""
  echo "${sep}"
  echo "${green}${bold}  ✅  ¡Listo!${nc}"
  echo "${sep}"
  echo ""
}
```

