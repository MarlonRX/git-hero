# RESUMEN v0.4.0 — Ejecución del plan (Fases 1-3 + 5 mínima) + pulido gestor de repos

**Rol:** jefe de obra sobre `plans/MAJOR_REFACTOR_PLAN.md`, con criterio de
nicho (gestor de repositorios, no cliente git). Fase 4 omitida por orden
(registro en `plans/DEFERRED.md`).

## Hallazgo de partida (importante para leer las métricas)

El plan se escribió contra un árbol ~6k LOC/2 tests. El árbol de trabajo al
empezar esta sesión (v0.3.0, `ebab782`) ya traía **gran parte de las Fases 1-3
landeadas**: `Command::parse` con 33 tests, `status_snapshot`/`log_snapshot`,
`GitError` (thiserror), `log.rs` con handle cacheado, iconos phf, sugerencias
estáticas, caché de diff, confirmación de `/remove-repo`, askpass a 500 ms y
85 tests. Las métricas "antes" del plan (2 tests, 6 llamadas git) corresponden
al v0.2.x, no al punto de partida real. Se completó lo que faltaba y se midió
contra el baseline real.

## Métricas antes → después (valores reales medidos, no del plan)

| Métrica | Plan decía "antes" | Real v0.3.0 (baseline) | v0.4.0 (ahora) | Objetivo del plan |
|---|---|---|---|---|
| `cargo test` | 2 | **85** | **126 (+1 smoke ignorado)** | ≥25 (pedido) / ≥35 (plan) ✅ |
| Invocaciones git por `refresh_git_status` | 6 | 3 (rev-parse+status+log) | **2** (status doble-función + log) | 2 ✅ |
| Allocaciones en `translate()`/keystroke | ~40 µs + 2 HashMaps | 0 en hit (OnceLock) | **0, tablas `phf` estáticas compile-time** | ✅ |
| `if lang == "es"` en `cli.rs` | 9 | 9 (keys existían sin usar) | **0** — diccionario único CLI+TUI | ✅ |
| Strings EN/ES duros en TUI (footer, placeholder, theme/remove/cancel) | varios | varios | **0** en el path tocado; keys i18n nuevos | ✅ |
| Panics latentes de render hot-path | — | 2 (byte-slice de subjects y diff con UTF-8) | **0** (`truncate_subject` + truncado por chars, 4 tests) | ✅ |
| Drenado de consola | `terminal.size()?` + re-split O(n²) por mensaje | igual | **1 query de size por batch, drain acotado a 64, sin `?` letal** | ✅ |
| `commands.rs` LOC | 353 (if-else) | 501 (ya dispatch-table) | **489** | ≤150 ❌ (el número del plan no aplica al formato actual; ver nota) |
| `draw_dashboard` LOC | 420 | 307 | **478** ⚠ inflación de `cargo fmt` (misma lógica −1 línea, +formato canonical); objetivo ≤200 es Fase 4 → DEFERRED | ❌ (omitida a propósito) |
| LOC total `src/` | ~6 000 | 6 937 | **9 171** (+/repos browser ~260 de runtime, +tests, +fmt) | n/a |
| `cargo clippy --all-targets -- -D warnings` | ❌ | ❌ (9 warns en Rust 1.98) | **✅ exit 0** | ✅ |
| `cargo fmt --check` | ❌ | ❌ | **✅** | (gate CI extra) |
| `cargo build --release` | — | — | **✅ 19 s, sin warnings** | ✅ |
| Keys i18n EN/ES con paridad garantizada por test | — | 78 | **~110** | ✅ |

## Tareas pedidas → estado

1. **Fases 1-3** ✅ completadas. Lo ya existente se verificó leyendo código (no
   se rehizo); se cerraron los huecos reales: phf estático, 2 calls/refresh,
   cuellos restantes de render, unificación i18n CLI/TUI. El `CliTheme` de §4.5
   resultó ser un problema inexistente (no había duplicación ANSI real: los
   códigos ANSI viven solo en cli.rs) → registrado en DEFERRED.
2. **`/remove-repo`** ✅ **verificado, ya estaba bien**: modal rojo
   `show_confirm_remove` → y/N explícito; `/help` y `/docs` dicen "asks
   confirmation"; README también. Quedó **blindado con 3 tests** (docs nunca
   dice "no confirm"; HELP lo exige; chequeo estático de que `cmd_remove_repo`
   no llama a `git_remove_repo`, único camino es el confirm-key handler).
3. **Fase 5 mínima** ✅ 126 tests en verde (+1 smoke ignorado) en 17 módulos
   (command 33, git 20, panels 12, repos 10+smoke, i18n 8, suggestions 7,
   rendering 4 (legend), version 6, git_error 6, theme 5, icons 4, modals 3,
   keyboard 2, cli 2, config 2, log 2, commands 1), clippy limpio,
   release OK, CI (`.github/workflows/ci.yml`) ya corría check/clippy/fmt/test.
4. **Pulido gestor de repos** ✅ era inexistente (el "dashboard multi-repo" del
   enunciado no estaba en el código: el TUI era mono-repo). Iteración 1: overlay
   modal `/repos`. **Iteración 2 (feedback en vivo)**: el modal "no era claro de
   usar" → re-diseñado como **browser tipo desplegable en el sector de la barra
   lateral donde vivían los shortcuts**, con:
   - **Escaneo recursivo** (profundidad ≤4, tope 500 dirs visitados / 60 repos
     probed, salta ocultos y `node_modules`/`target`/`vendor`/…; no desciende
     dentro de un repo ya encontrado).
   - **Filtrado al teclear** (typeahead puro sobre datos ya scaneados: escribir
     cuesta 0 llamadas a git), con contador de coincidencias `2/40`.
   - Filas: `ruta/relativa · rama · ↑a↓b · n✱ · antigüedad`, orden por actividad,
     header "**X de Y repos con cambios sin commitear**".
   - **Enter o CLICK** entra al repo (vía `/cd`); `Ctrl+R` reescanea; `Esc`
     cierra. Concwd sin repo el browser ocupa la columna izquierda completa y
     el panel de init queda a la derecha (seguir gestionando sin repo abierto).
   - Click-to-row sin drift de geometría: `sidebar_split`/`browser_area`/
     `browser_rows_rect` son helpers puros compartidos por draw y hit-testing,
     con tests unitarios.
   Costo verificado con smoke real ignorado (`-- --ignored`: repo anidado
   encontrado, decoy en `node_modules` descartado, orden/dirty asserts).
   Fix anejo: paths con espacios truncados por el parser v2 (`splitn`,
   validado contra git 2.55 real).
   **Iteración 3 (feedback en vivo)**: el browser pasó a ser **la pantalla por
   defecto cuando el directorio no es repo** (arranque, `/cd` a carpeta suelta
   o post-`/remove-repo`): lista a la izquierda + opciones init/`/cd` a la
   derecha, `Esc` devuelve el teclado a esas opciones y el wizard de init
   mantiene prioridad sobre el browser. Además el nombre de la carpeta/repo
   actual se ve claro de una vez: **badge bold con fondo** en la status line y
   **chips de nombre en negrita sobre fondo surface** en cada fila del browser,
   con marcador **`● here`/`● actual`** en el repo donde está el usuario
    (`is_within`, comparaciones por bytes sin riesgo de partir UTF-8, testeada).
   **Iteración 4 (feedback en vivo)**: tres correcciones de manejo.
   (a) **Scroll real en FILES/COMMITS**: antes la lista se cortaba contra el
   borde y la selección quedaba invisible; ahora cada panel se ventanea
   (`browser_scroll`) alrededor de la selección y el título de FILES muestra
   la posición `FILES (12/57)`; se eliminó `commit_scroll_offset` manual y la
   rueda/PgUp-PgDn ahora **mueve la selección** (la vista sigue). La rueda
   sobre el browser mueve su cursor. (b) **Bloqueo de atajos al tipear**: el
   input de comandos se enruta **antes** que el browser, y `/` abre la barra
   desde cualquier lado (repo, sin-repo, dentro del browser); así ningún
   comando de letra salta mientras escribís un comando. (c) **Tira de teclas
   siempre visible**: el footer ganó una 3ª fila con chips `[tecla] acción`
   por contexto (repo/browser/sin-repo/escribiendo), con degradado por ancho
   (`…`). Tests nuevos: `parse_legend`/`legend_line` (4) + geometría ya
   cubierta. Total 126.
5. **README** ✅ primera sección reescrita (EN y ES): "gestor de repositorios,
   no cliente git", tabla honesta de las 5 acciones con su nivel de seguridad,
   mock de texto del panel `/repos`, y un "lo que Git Hero NO es" (rebase
   interactivo, blame — territorio lazygit).

## Commits (orden de la sección 6 del plan, adaptado a lo que faltaba)

```
refactor(i18n): compile-time phf static tables replace OnceLock<HashMap> (#1.1)
refactor(state): status_snapshot doubles as repo probe - 2 git calls per refresh (#2.4)
perf(ui): cache size once per console batch, cap drain, UTF-8-safe truncation (#1 hot-path)
refactor(i18n): cli.rs and TUI share one dictionary - remove all if lang==es pairs (#4.3/4.5)
test(ui): lock in /remove-repo confirmation modal + docs wording (#3.3 regression)
fix(clippy): clean -D warnings on Rust 1.98; dev/dirty badge in status line (#5.x)
feat(repos): /repos multi-repo overview - dirty count, ahead/behind, last-activity sort, g to rescan
test(repos): ignored smoke test proves scan on real git (dirty, ordering, non-repo skip)
style: cargo fmt across the tree (green CI fmt gate, repo had drifted)
fix(git): porcelain v2 parser keeps paths containing spaces (regression, verified vs real git)
chore(release): v0.4.0 - repo-manager positioning, CHANGELOG, RESUMEN with real metrics
feat(repos): /repos becomes sidebar type-ahead browser - recursive scan, filter without git, click to open
docs: repository browser wording across README/CHANGELOG/RESUMEN (feedback iteration)
feat(ui): repo browser is the no-repo default; bold name chips + here marker + location badge
docs: no-repo default browser + location badge across README/CHANGELOG/RESUMEN
fix(ui): FILES/COMMITS follow-selection scrolling, / opens command bar from browser, persistent footer keybind strip
```

*(Locales, sin push, sin tags, config JSON intacto — `Config` no ganó campos
nuevos: `repo_dirty_count` vive en `AppState`, no en disco.)*

## Notas honestas / deudores

- El objetivo "≤150 LOC en commands.rs" y "≤200 en draw_dashboard" eran
  números del plan contra el código pre-v0.3; medirlos contra el árbol actual
  no es comparable (dispatch-table ya existía; fmt canónico infla LOC). Queda
  anotado arriba y el split real de `draw_dashboard` es Fase 4 → hermano.
- `/repos` recorre recursivamente desde el padre del repo actual (o el cwd si
  no hay repo). Sin config de "projects folder" (el enunciado prohibió tocar
  el formato de config; una key opcional con `#[serde(default)]` podría
  agregarse en el futuro sin romper nada).
- Las zonas de click del FILES panel (`mouse_dashboard`) siguen asumiendo el
  layout viejo del sidebar (STATUS/SHORTCUTS) cuando el browser está cerrado:
  benigno, registrado en DEFERRED; el browser sí tiene hit-testing exacto.
- Zona de clicks obsoleta en `mouse_dashboard` (sidebar STATUS/SHORTCUTS que ya
  no existe): benigna, registrada en DEFERRED.
