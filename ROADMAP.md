# Hoja de Ruta (Roadmap) - gacp-rx 🗺️

Este documento describe el plan paso a paso para desarrollar **gacp-rx** en **Rust**, progresando desde una base de comandos hasta una interfaz TUI completa.

---

## 📅 Fases de Desarrollo

### 🏁 Fase 1: Inicialización del Proyecto y Motor de Git
El objetivo es configurar el entorno en Rust y portar la lógica de verificación de Git de tu script de zsh a Rust (ejecución de comandos, parseo de diferencias).

1.  **Inicialización:**
    *   Ejecutar `cargo init` en la carpeta `gacp-rx`.
    *   Configurar `Cargo.toml` con las dependencias necesarias:
        ```toml
        [dependencies]
        ratatui = "0.26"
        crossterm = { version = "0.27", features = ["event-stream"] }
        tokio = { version = "1", features = ["full"] } # Para tareas asíncronas
        ```
2.  **Módulo Git (`git.rs`):**
    *   Implementar funciones para invocar comandos Git de manera segura:
        *   `git rev-parse --is-inside-work-tree` (validar repo).
        *   `git symbolic-ref --short HEAD` (obtener rama actual).
        *   `git rev-list --count HEAD..origin/branch` (contar commits detrás).
        *   `git status --porcelain` (obtener lista de archivos modificados).
    *   Estructurar los datos de salida en un struct `GitStatus`.

---

### 🖥️ Fase 2: Ciclo de Eventos y Terminal (`event.rs` & `main.rs`)
Configurar el modo raw de la terminal, la captura de teclado y el loop principal de dibujado.

1.  **Configuración de Terminal:**
    *   Activar `crossterm::terminal::enable_raw_mode`.
    *   Configurar el bucle principal (`main loop`) de renderizado de Ratatui.
2.  **Manejador de Eventos:**
    *   Capturar eventos de teclado (`Esc` para salir, `Flechas` para navegar, `Espacio` para interactuar, `c` para commit).
    *   Asegurar un cierre limpio de la terminal restaurando el modo original incluso si la aplicación falla (usando pánicos o destructores).

---

### 🎨 Fase 3: Interfaz Visual Básica (`ui.rs` & `app.rs`)
Crear el diseño visual básico usando los bloques (*Layouts* y *Paragraphs*) de Ratatui.

1.  **Diseño de Layout:**
    *   Dividir la pantalla en 3 secciones principales:
        *   **Header:** Información del repositorio, rama actual y estado remoto (Commits Ahead / Behind).
        *   **Body (Izquierda):** Lista interactiva de archivos modificados/no rastreados.
        *   **Body (Derecha):** Detalle o preview del archivo seleccionado (diferencias / git diff).
        *   **Footer:** Leyenda de atajos de teclado (`[c] Commit`, `[u] Pull`, `[p] Push`, `[q] Salir`).
2.  **Estado de la Aplicación (`app.rs`):**
    *   Definir el struct `App` que almacena la lista de archivos, el índice del archivo seleccionado actualmente y el estado de carga (fetching).

---

### 🔄 Fase 4: Interactividad (Stage/Unstage de Archivos)
Permitir al usuario decidir qué archivos añadir al commit de forma visual.

1.  **Staging Visual:**
    *   Mostrar un indicador visual al lado de cada archivo (por ejemplo, `[ ]` para sin stage, `[x]` para staged).
    *   Al presionar `Espacio` sobre un archivo:
        *   Si no está en stage: ejecutar `git add <archivo>`.
        *   Si está en stage: ejecutar `git reset HEAD <archivo>`.
    *   Refrescar el estado de Git inmediatamente después de cada acción.

---

### 💬 Fase 5: Modal de Commit
Implementar una caja de diálogo interactiva para ingresar el mensaje de commit.

1.  **Modal Pop-up:**
    *   Al presionar `c` (si hay cambios en staging):
        *   Dibujar un modal flotante sobrepuesto en el centro de la pantalla.
        *   Habilitar entrada de texto dinámica.
2.  **Acción de Commit:**
    *   Al presionar `Enter` en el modal: ejecutar `git commit -m "<mensaje>"` usando el texto ingresado.
    *   Cerrar el modal y refrescar el estado del repositorio.

---

### 🚀 Fase 6: Acciones de Sincronización (Pull & Push)
Manejar la descarga y subida de cambios de forma interactiva y asíncrona para no congelar la UI.

1.  **Flujo de Pull/Push:**
    *   Si hay commits por recibir (detrás), el indicador se muestra en amarillo y habilitar la tecla `u`.
    *   Si hay commits por enviar (adelante), habilitar la tecla `p`.
2.  **Operación Asíncrona:**
    *   Ejecutar las operaciones de red (`git pull`/`git push`) en un hilo secundario utilizando `tokio::spawn` para mantener la interfaz fluida e interactiva (mostrando un spinner o mensaje de "Cargando...").

---

## 🛠️ Cómo Empezar Hoy mismo

1.  Asegúrate de tener Rust instalado (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).
2.  Crea la estructura del proyecto en Rust ejecutando en la carpeta del repositorio:
    ```bash
    cargo init
    ```
3.  Copia las dependencias de la **Fase 1** a tu `Cargo.toml`.
4.  Empieza creando el archivo [src/git.rs](file:///Users/mramirez/Projects/gacp-rx/src/git.rs) para portar tu script actual a comandos ejecutados desde Rust.
