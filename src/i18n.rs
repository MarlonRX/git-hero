//! Internationalization (i18n) — English and Spanish translation strings.
//!
//! Translations live in compile-time [`phf::Map`] tables: zero runtime
//! construction, zero allocations on lookup. Each lookup returns a
//! `Cow<'static, str>` — borrowed from the static table when the key is
//! found, and one allocation (the key itself) when the key is missing so
//! typos are immediately visible to the user.

use std::borrow::Cow;

use phf::Map;

/// English dictionary (built at compile time).
static EN_DICT: Map<&'static str, &'static str> = phf::phf_map! {
    "setup_title" => "Git Hero — First Time Setup",
    "setup_lang" => "Select Language:",
    "setup_icons" => "Select Icon Set (Nerd Fonts require compatible terminal font):",
    "setup_theme" => "Select Initial Theme:",
    "repo_status" => "Repository Status",
    "not_git_repo" => "NOT A GIT REPOSITORY",
    "suggest_cd" => "Type /cd <path> to switch to a valid repository.",
    "status_ready" => "Ready. Type / to enter a command or /help to see commands.",
    "status_fetching" => "Fetching remote updates...",
    "status_pulling" => "Pulling remote updates...",
    "status_pushing" => "Pushing commits to remote...",
    "status_success" => "Operation completed successfully!",
    "status_not_git" => "Cannot execute: Not inside a Git repository.",
    "status_change_dir_err" => "Error changing directory: {}",
    "status_commit_success" => "Commit created successfully!",
    "theme_title" => "Select Theme",
    "footer_status" => "Status",
    "commands_list" => "Commands: /cd <path>, /fetch, /pull, /push, /commit <msg>, /themes, /quit",
    "welcome_message" => "Welcome to Git Hero!",
    "help_header" => "Available Commands",
    "setup_help" => "Press [Up/Down] to select, [Enter] to confirm.",
    "theme_help" => "Press [Up/Down] to select, [Enter] to save, [Esc] to cancel.",
    "status_stage_all_ok" => "Staged all changes.",
    "status_unstage_all_ok" => "Unstaged all changes.",
    "status_undo_commit" => "Undoing last commit (soft reset)...",
    "status_undo_commit_ok" => "Last commit undone. Changes kept in working tree.",
    "status_remove_ok" => "Git repository removed. .git directory deleted.",
    "status_branch_list" => "Branch list loaded. See diff panel.",
    "status_config_list" => "Config loaded. See diff panel.",
    "status_stash_ok" => "Changes stashed.",
    "status_stash_pop_ok" => "Stash popped. Changes restored.",
    // ── CLI + TUI parity keys (unified via translate) ─────────────
    "status_cd_ok" => "Changed directory to: {}",
    "status_cd_err" => "Error changing directory: {}",
    "status_remote_set" => "Remote origin → {}",
    "status_branch_deleted" => "Branch '{}' deleted.",
    "status_branch_switched" => "Switched to branch: {}",
    "status_branch_created" => "Created and switched to branch: {}",
    "status_config_set_local" => "Config set: {} = {}",
    "status_config_set_global" => "Global config set: {} = {}",
    "status_config_get_local" => "Local: {} = {}",
    "status_config_get_global" => "Global: {} = {}",
    "status_config_not_found" => "Config key '{}' not found.",
    "status_theme_changed" => "Theme changed to: {}",
    "status_cmd_already_running" => "A command is already running.",
    "status_remove_cancelled" => "Repository removal cancelled.",
    "status_err_commit_empty" => "Error: commit message empty.",
    "status_err_commit" => "Error committing: {}",
    "status_err_stage" => "Error staging all: {}",
    "status_err_unstage" => "Error unstaging all: {}",
    "status_err_undo_commit" => "Error undoing commit: {}",
    "status_err_remove_repo" => "Error removing repo: {}",
    "status_err_set_remote" => "Error setting remote: {}",
    "status_err_branch_delete" => "Error deleting branch: {}",
    "status_err_branch_create" => "Error creating branch: {}",
    "status_err_branch_switch" => "Error switching branch: {}",
    "status_err_list_branches" => "Error listing branches: {}",
    "status_err_list_config" => "Error listing config: {}",
    "status_err_config_get" => "Error reading config: {}",
    "status_err_config_set" => "Error setting config: {}",
    "status_err_config_set_global" => "Error setting global config: {}",
    "status_err_stash" => "Error stashing: {}",
    "status_err_stash_pop" => "Error popping stash: {}",
    "status_usage_remote" => "Usage: /remote <url> - adds or updates origin remote",
    "status_usage_config" => "Usage: /config <key> [value]",
    "status_usage_config_global" => "Usage: /config-global <key> [value]",
    "ahead_label" => "Ahead",
    "behind_label" => "Behind",
    "no_changes_to_commit" => "No changes for commit",
    "no_pending_changes" => "No pending changes for commit",
    "commit_message_label" => "Commit message",
    "empty_message_cancelled" => "Empty message, cancelled",
    "remote_has_changes" => "Remote has changes",
    "do_pull" => "Do pull?",
    "pull_completed" => "Pull completed",
    "do_push" => "Do push?",
    "push_completed" => "Push completed",
    "local_commits_kept" => "Local commits kept, no push",
    "up_to_date_with_remote" => "You are up to date with remote",
    "fetch_complete" => "✓ Fetch complete.",
    // ── CLI-only messages (were hardcoded `if lang == "es"` pairs) ──
    "cli_err_pull" => "Error pulling: {}",
    "cli_err_push" => "Error pushing: {}",
    "cli_not_repo" => "Not inside a git repository",
    "git_flow_title" => "GIT FLOW",
    // ── TUI footer / input bar (were hardcoded EN/ES literals) ───
    "footer_help" => "? Help | q Quit",
    "input_placeholder" => "Type a command...",
    "modal_close_hint" => "Press any key to close.",
    // ── Update modal keys ─────────────────────────────────────────
    "update_title" => "🚀 Update Available",
    "update_new_version" => "A new version of Git Hero is available: v{}",
    "update_current_version" => "Current version: v{}",
    "update_yes" => "Open download page",
    "update_no" => "Remind me later",
    "update_skip" => "Don't show again for this version",
    // ── /language command keys ────────────────────────────────────
    "status_language_changed" => "Language changed to: {}",
    "status_language_same" => "Language is already {}",
    // ── Repo-overview (multi-repo manager) keys ───────────────────
    "repo_overview_title" => " Repository Overview ",
    "repo_overview_help" => "[↑/↓] Select  [Enter] Open  [g] Rescan  [Esc] Close",
    "repo_overview_summary" => "{} of {} repos have uncommitted changes",
    "repo_overview_empty" => "No Git repositories found near this directory.",
    "repo_overview_no_commits" => "no commits",
};

/// Spanish dictionary (built at compile time).
static ES_DICT: Map<&'static str, &'static str> = phf::phf_map! {
    "setup_title" => "Git Hero — Configuración Inicial",
    "setup_lang" => "Selecciona el Idioma:",
    "setup_icons" => "Selecciona el Set de Iconos:",
    "setup_theme" => "Selecciona el Tema Inicial:",
    "repo_status" => "Estado del Repositorio",
    "not_git_repo" => "NO ES UN REPOSITORIO GIT",
    "suggest_cd" => "Escribe /cd <ruta> para cambiar a un repositorio válido.",
    "status_ready" => "Listo. Escribe / para ingresar un comando o /help para verlos.",
    "status_fetching" => "Descargando actualizaciones del remoto...",
    "status_pulling" => "Descargando y fusionando cambios remotos...",
    "status_pushing" => "Subiendo commits al remoto...",
    "status_success" => "¡Operación completada con éxito!",
    "status_not_git" => "No se puede ejecutar: No estás en un repositorio Git.",
    "status_change_dir_err" => "Error al cambiar de directorio: {}",
    "status_commit_success" => "¡Commit creado con éxito!",
    "theme_title" => "Seleccionar Tema",
    "footer_status" => "Estado",
    "commands_list" => "Comandos: /cd <ruta>, /fetch, /pull, /push, /commit <msg>, /themes, /quit",
    "welcome_message" => "¡Bienvenido a Git Hero!",
    "help_header" => "Comandos Disponibles",
    "setup_help" => "Presiona [Arriba/Abajo] para elegir, [Enter] para confirmar.",
    "theme_help" => "Presiona [Arriba/Abajo] para elegir, [Enter] para guardar, [Esc] para cancelar.",
    "status_stage_all_ok" => "Todos los cambios preparados (staged).",
    "status_unstage_all_ok" => "Todos los cambios quitados (unstaged).",
    "status_undo_commit" => "Deshaciendo último commit (soft reset)...",
    "status_undo_commit_ok" => "Último commit deshecho. Cambios conservados.",
    "status_remove_ok" => "Repositorio Git eliminado. Directorio .git borrado.",
    "status_branch_list" => "Lista de ramas cargada. Ver panel de diff.",
    "status_config_list" => "Configuración cargada. Ver panel de diff.",
    "status_stash_ok" => "Cambios guardados en stash.",
    "status_stash_pop_ok" => "Stash recuperado. Cambios restaurados.",
    "status_cd_ok" => "Cambiado al directorio: {}",
    "status_cd_err" => "Error al cambiar de directorio: {}",
    "status_remote_set" => "Remoto origin → {}",
    "status_branch_deleted" => "Rama '{}' eliminada.",
    "status_branch_switched" => "Cambiado a la rama: {}",
    "status_branch_created" => "Rama '{}' creada y activada.",
    "status_config_set_local" => "Config local: {} = {}",
    "status_config_set_global" => "Config global: {} = {}",
    "status_config_get_local" => "Local: {} = {}",
    "status_config_get_global" => "Global: {} = {}",
    "status_config_not_found" => "Clave de config '{}' no encontrada.",
    "status_theme_changed" => "Tema cambiado a: {}",
    "status_cmd_already_running" => "Ya hay un comando en ejecución.",
    "status_remove_cancelled" => "Eliminación del repositorio cancelada.",
    "status_err_commit_empty" => "Error: mensaje de commit vacío.",
    "status_err_commit" => "Error al hacer commit: {}",
    "status_err_stage" => "Error al stagear todo: {}",
    "status_err_unstage" => "Error al unstagear todo: {}",
    "status_err_undo_commit" => "Error al deshacer commit: {}",
    "status_err_remove_repo" => "Error al eliminar repo: {}",
    "status_err_set_remote" => "Error al configurar remoto: {}",
    "status_err_branch_delete" => "Error al eliminar rama: {}",
    "status_err_branch_create" => "Error al crear rama: {}",
    "status_err_branch_switch" => "Error al cambiar de rama: {}",
    "status_err_list_branches" => "Error al listar ramas: {}",
    "status_err_list_config" => "Error al listar config: {}",
    "status_err_config_get" => "Error al leer config: {}",
    "status_err_config_set" => "Error al configurar: {}",
    "status_err_config_set_global" => "Error al configurar global: {}",
    "status_err_stash" => "Error al guardar stash: {}",
    "status_err_stash_pop" => "Error al recuperar stash: {}",
    "status_usage_remote" => "Uso: /remote <url> - añade o actualiza el remoto origin",
    "status_usage_config" => "Uso: /config <clave> [valor]",
    "status_usage_config_global" => "Uso: /config-global <clave> [valor]",
    "ahead_label" => "Adelante",
    "behind_label" => "Detrás",
    "no_changes_to_commit" => "Sin cambios para commit",
    "no_pending_changes" => "Sin cambios pendientes para commit",
    "commit_message_label" => "Mensaje del commit",
    "empty_message_cancelled" => "Mensaje vacío, cancelado",
    "remote_has_changes" => "El remoto tiene cambios",
    "do_pull" => "¿Hacer pull?",
    "pull_completed" => "Pull completado",
    "do_push" => "¿Hacer push?",
    "push_completed" => "Push completado",
    "local_commits_kept" => "Commits locales guardados, sin push",
    "up_to_date_with_remote" => "Estás al día con el remoto",
    "fetch_complete" => "✓ Fetch completo.",
    "cli_err_pull" => "Error al hacer pull: {}",
    "cli_err_push" => "Error al hacer push: {}",
    "cli_not_repo" => "No estás dentro de un repositorio git",
    "git_flow_title" => "FLUJO GIT",
    "footer_help" => "? Ayuda | q Salir",
    "input_placeholder" => "Escribe un comando...",
    "modal_close_hint" => "Presiona cualquier tecla para cerrar.",
    "update_title" => "🚀 Actualización Disponible",
    "update_new_version" => "Una nueva versión de Git Hero está disponible: v{}",
    "update_current_version" => "Versión actual: v{}",
    "update_yes" => "Abrir página de descarga",
    "update_no" => "Recordarme después",
    "update_skip" => "No volver a mostrar esta versión",
    "status_language_changed" => "Idioma cambiado a: {}",
    "status_language_same" => "El idioma ya es {}",
    "repo_overview_title" => " Resumen de Repositorios ",
    "repo_overview_help" => "[↑/↓] Seleccionar  [Enter] Abrir  [g] Reescanear  [Esc] Cerrar",
    "repo_overview_summary" => "{} de {} repos tienen cambios sin commitear",
    "repo_overview_empty" => "No se encontraron repositorios Git cerca de este directorio.",
    "repo_overview_no_commits" => "sin commits",
};

/// Look up a translation for `key` in the given language.
///
/// Returns `Cow::Borrowed` (no allocation) when the key is present in the
/// dictionary, and `Cow::Owned(key.to_string())` (one allocation) when the
/// key is missing — the key itself is shown to the user as a fallback so
/// typos are immediately visible.
pub fn translate(lang: &str, key: &str) -> Cow<'static, str> {
    let dict = if lang == "es" { &ES_DICT } else { &EN_DICT };
    match dict.get(key) {
        Some(value) => Cow::Borrowed(*value),
        None => Cow::Owned(key.to_string()),
    }
}

/// Look up `key` and substitute `{}` placeholders with `args` (left to
/// right). Returns an owned `String` because the result cannot borrow
/// from the caller's args. Always one allocation for the result; the
/// per-placeholder replace is in-place.
pub fn trf(lang: &str, key: &str, args: &[&str]) -> String {
    let mut s = translate(lang, key).into_owned();
    for arg in args {
        s = s.replacen("{}", arg, 1);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_english_for_unknown_lang() {
        assert_eq!(translate("xx", "setup_title"), "Git Hero — First Time Setup");
        assert_eq!(translate("", "setup_title"), "Git Hero — First Time Setup");
    }

    #[test]
    fn returns_spanish_for_es() {
        assert_eq!(
            translate("es", "setup_title"),
            "Git Hero — Configuración Inicial"
        );
    }

    #[test]
    fn fallback_returns_key_when_missing() {
        let result = translate("en", "nonexistent_key_xyz");
        assert_eq!(result, "nonexistent_key_xyz");
        assert!(matches!(result, Cow::Owned(_)));
    }

    #[test]
    fn known_key_is_borrowed_zero_alloc() {
        let result = translate("en", "setup_title");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn all_keys_exist_in_both_languages() {
        for key in EN_DICT.keys() {
            assert!(
                ES_DICT.contains_key(key),
                "Spanish dict is missing key: {key}"
            );
        }
        for key in ES_DICT.keys() {
            assert!(
                EN_DICT.contains_key(key),
                "English dict is missing key: {key}"
            );
        }
    }

    #[test]
    fn trf_substitutes_placeholders_left_to_right() {
        assert_eq!(trf("en", "status_config_set_local", &["a.b", "c"]), "Config set: a.b = c");
        // Two placeholders with distinct args.
        assert_eq!(trf("es", "status_config_get_global", &["k", "v"]), "Global: k = v");
    }

    #[test]
    fn trf_with_no_args_is_passthrough() {
        assert_eq!(trf("en", "status_success", &[]), "Operation completed successfully!");
    }

    #[test]
    fn cli_and_tui_parity_keys_present() {
        // The keys migrated out of `if lang == "es"` blocks in cli.rs must exist.
        for key in [
            "ahead_label",
            "behind_label",
            "no_changes_to_commit",
            "no_pending_changes",
            "commit_message_label",
            "empty_message_cancelled",
            "remote_has_changes",
            "do_pull",
            "pull_completed",
            "do_push",
            "push_completed",
            "local_commits_kept",
            "up_to_date_with_remote",
        ] {
            assert!(EN_DICT.contains_key(key), "EN missing {key}");
            assert!(ES_DICT.contains_key(key), "ES missing {key}");
        }
    }
}
