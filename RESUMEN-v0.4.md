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
| `cargo test` | 2 | **85** | **112 (+1 smoke ignorado)** | ≥25 (pedido) / ≥35 (plan) ✅ |
| Invocaciones git por `refresh_git_status` | 6 | 3 (rev-parse+status+log) | **2** (status doble-función + log) | 2 ✅ |
| Allocaciones en `translate()`/keystroke | ~40 µs + 2 HashMaps | 0 en hit (OnceLock) | **0, tablas `phf` estáticas compile-time** | ✅ |
| `if lang == "es"` en `cli.rs` | 9 | 9 (keys existían sin usar) | **0** — diccionario único CLI+TUI | ✅ |
| Strings EN/ES duros en TUI (footer, placeholder, theme/remove/cancel) | varios | varios | **0** en el path tocado; keys i18n nuevos | ✅ |
| Panics latentes de render hot-path | — | 2 (byte-slice de subjects y diff con UTF-8) | **0** (`truncate_subject` + truncado por chars, 4 tests) | ✅ |
| Drenado de consola | `terminal.size()?` + re-split O(n²) por mensaje | igual | **1 query de size por batch, drain acotado a 64, sin `?` letal** | ✅ |
| `commands.rs` LOC | 353 (if-else) | 501 (ya dispatch-table) | **487** | ≤150 ❌ (el número del plan no aplica al formato actual; ver nota) |
| `draw_dashboard` LOC | 420 | 307 | **467** ⚠ inflación de `cargo fmt` (misma lógica −1 línea, +formato canonical); objetivo ≤200 es Fase 4 → DEFERRED | ❌ (omitida a propósito) |
| LOC total `src/` | ~6 000 | 6 937 | **8 898** (+/repos 350, +tests, +fmt) | n/a |
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
3. **Fase 5 mínima** ✅ 112 tests en 16 módulos (git 20, command 33, i18n 8,
   repos 9, theme 5, config 2, panels 4, icons 4, keyboard 2, cli 2, modals 3,
   suggestions 7, version 6, git_error 6, log 2, commands 1), clippy limpio,
   release OK, CI (`.github/workflows/ci.yml`) ya corría check/clippy/fmt/test.
4. **Pulido gestor de repos** ✅ era inexistente (el "dashboard multi-repo" del
   enunciado no estaba en el código: el TUI era mono-repo). Se construyó lo
   mínimo del nicho sin pasar el techo de ~300 líneas de runtime:
   `/repos` o tecla `g` → **Repository Overview**: una fila por repo hermano
   (rama, ↑ahead/↓behind, **contador de archivos sucios**, edad del último
   commit), **ordenado por última actividad**, header "**n de m repos tienen
   cambios sin commitear**", `g` = **refresco global**, Enter = saltar al repo
   (vía `/cd`). Costo: 2 procesos git por repo, solo al abrir/reescanear.
   Verificado con smoke test real (`-- --ignored`: `git init` de 2 repos +
   asserts de orden/dirty/skip-no-repos). Fix anejo: paths con espacios
   truncados por el parser v2 (`splitn`, validado contra git 2.55 real).
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
chore(release): v0.4.0 - repo-manager positioning, CHANGELOG, RESUMEN
```

*(Locales, sin push, sin tags, config JSON intacto — `Config` no ganó campos
nuevos: `repo_dirty_count` vive en `AppState`, no en disco.)*

## Notas honestas / deudores

- El objetivo "≤150 LOC en commands.rs" y "≤200 en draw_dashboard" eran
  números del plan contra el código pre-v0.3; medirlos contra el árbol actual
  no es comparable (dispatch-table ya existía; fmt canónico infla LOC). Queda
  anotado arriba y el split real de `draw_dashboard` es Fase 4 → hermano.
- `/repos` lista hijos del cwd si NO hay repo, y hermanos si el cwd ES repo.
  Sin config de "projects folder" (el enunciado prohibió tocar el formato de
  config; una key opcional podría agregarse en el futuro sin romper nada).
- Click para seleccionar fila en `/repos`: no hecho (keyboard completo).
- Zona de clicks obsoleta en `mouse_dashboard` (sidebar STATUS/SHORTCUTS que ya
  no existe): benigna, registrada en DEFERRED.
