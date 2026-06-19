//! Internationalization (i18n) — English and Spanish translation strings.
//!
//! Translations are stored in `OnceLock<HashMap>` so the dictionaries are built
//! once on first use and reused for the lifetime of the program. Each lookup
//! returns a `Cow<'static, str>`: zero allocations when the key is found
//! (borrowed from the static dict) and one allocation when the key is missing
//! (the key itself is returned as a fallback).

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

type Dict = HashMap<&'static str, &'static str>;

static EN_DICT: OnceLock<Dict> = OnceLock::new();
static ES_DICT: OnceLock<Dict> = OnceLock::new();

fn en_dict() -> &'static Dict {
    EN_DICT.get_or_init(|| {
        let mut m = Dict::with_capacity(32);
        m.insert("setup_title", "Git Hero — First Time Setup");
        m.insert("setup_lang", "Select Language:");
        m.insert("setup_icons", "Select Icon Set (Nerd Fonts require compatible terminal font):");
        m.insert("setup_theme", "Select Initial Theme:");
        m.insert("repo_status", "Repository Status");
        m.insert("not_git_repo", "NOT A GIT REPOSITORY");
        m.insert("suggest_cd", "Type /cd <path> to switch to a valid repository.");
        m.insert("status_ready", "Ready. Type / to enter a command or /help to see commands.");
        m.insert("status_fetching", "Fetching remote updates...");
        m.insert("status_pulling", "Pulling remote updates...");
        m.insert("status_pushing", "Pushing commits to remote...");
        m.insert("status_success", "Operation completed successfully!");
        m.insert("status_not_git", "Cannot execute: Not inside a Git repository.");
        m.insert("status_change_dir_err", "Error changing directory: {}");
        m.insert("status_commit_success", "Commit created successfully!");
        m.insert("theme_title", "Select Theme");
        m.insert("footer_status", "Status");
        m.insert("commands_list", "Commands: /cd <path>, /fetch, /pull, /push, /commit <msg>, /themes, /quit");
        m.insert("welcome_message", "Welcome to Git Hero!");
        m.insert("help_header", "Available Commands");
        m.insert("setup_help", "Press [Up/Down] to select, [Enter] to confirm.");
        m.insert("theme_help", "Press [Up/Down] to select, [Enter] to save, [Esc] to cancel.");
        m.insert("status_stage_all_ok", "Staged all changes.");
        m.insert("status_unstage_all_ok", "Unstaged all changes.");
        m.insert("status_undo_commit", "Undoing last commit (soft reset)...");
        m.insert("status_undo_commit_ok", "Last commit undone. Changes kept in working tree.");
        m.insert("status_remove_ok", "Git repository removed. .git directory deleted.");
        m.insert("status_branch_list", "Branch list loaded. See diff panel.");
        m.insert("status_config_list", "Config loaded. See diff panel.");
        m.insert("status_stash_ok", "Changes stashed.");
        m.insert("status_stash_pop_ok", "Stash popped. Changes restored.");
        // ── New keys for Phase 4 i18n refactor (CLI + TUI parity) ──
        m.insert("status_cd_ok", "Changed directory to: {}");
        m.insert("status_cd_err", "Error changing directory: {}");
        m.insert("status_remote_set", "Remote origin → {}");
        m.insert("status_branch_deleted", "Branch '{}' deleted.");
        m.insert("status_branch_switched", "Switched to branch: {}");
        m.insert("status_branch_created", "Created and switched to branch: {}");
        m.insert("status_config_set_local", "Config set: {} = {}");
        m.insert("status_config_set_global", "Global config set: {} = {}");
        m.insert("status_config_get_local", "Local: {} = {}");
        m.insert("status_config_get_global", "Global: {} = {}");
        m.insert("status_config_not_found", "Config key '{}' not found.");
        m.insert("status_theme_changed", "Theme changed to: {}");
        m.insert("status_cmd_already_running", "A command is already running.");
        m.insert("status_remove_cancelled", "Repository removal cancelled.");
        m.insert("status_err_commit_empty", "Error: commit message empty.");
        m.insert("status_err_commit", "Error committing: {}");
        m.insert("status_err_stage", "Error staging all: {}");
        m.insert("status_err_unstage", "Error unstaging all: {}");
        m.insert("status_err_undo_commit", "Error undoing commit: {}");
        m.insert("status_err_remove_repo", "Error removing repo: {}");
        m.insert("status_err_set_remote", "Error setting remote: {}");
        m.insert("status_err_branch_delete", "Error deleting branch: {}");
        m.insert("status_err_branch_create", "Error creating branch: {}");
        m.insert("status_err_branch_switch", "Error switching branch: {}");
        m.insert("status_err_list_branches", "Error listing branches: {}");
        m.insert("status_err_list_config", "Error listing config: {}");
        m.insert("status_err_config_get", "Error reading config: {}");
        m.insert("status_err_config_set", "Error setting config: {}");
        m.insert("status_err_config_set_global", "Error setting global config: {}");
        m.insert("status_err_stash", "Error stashing: {}");
        m.insert("status_err_stash_pop", "Error popping stash: {}");
        m.insert("status_usage_remote", "Usage: /remote <url> - adds or updates origin remote");
        m.insert("status_usage_config", "Usage: /config <key> [value]");
        m.insert("status_usage_config_global", "Usage: /config-global <key> [value]");
        m.insert("ahead_label", "Ahead");
        m.insert("behind_label", "Behind");
        m.insert("no_changes_to_commit", "No changes for commit");
        m.insert("no_pending_changes", "No pending changes for commit");
        m.insert("commit_message_label", "Commit message");
        m.insert("empty_message_cancelled", "Empty message, cancelled");
        m.insert("remote_has_changes", "Remote has changes");
        m.insert("do_pull", "Do pull?");
        m.insert("pull_completed", "Pull completed");
        m.insert("do_push", "Do push?");
        m.insert("push_completed", "Push completed");
        m.insert("local_commits_kept", "Local commits kept, no push");
        m.insert("up_to_date_with_remote", "You are up to date with remote");
        m.insert("fetch_complete", "✓ Fetch complete.");
        // ── Update modal keys ──────────────────────────────────
        m.insert("update_title", "🚀 Update Available");
        m.insert("update_new_version", "A new version of Git Hero is available: v{}");
        m.insert("update_current_version", "Current version: v{}");
        m.insert("update_yes", "Open download page");
        m.insert("update_no", "Remind me later");
        m.insert("update_skip", "Don't show again for this version");
        // ── /language command keys ──────────────────────────────
        m.insert("status_language_changed", "Language changed to: {}");
        m.insert("status_language_same", "Language is already {}");
        m
    })
}

fn es_dict() -> &'static Dict {
    ES_DICT.get_or_init(|| {
        let mut m = Dict::with_capacity(32);
        m.insert("setup_title", "Git Hero — Configuración Inicial");
        m.insert("setup_lang", "Selecciona el Idioma:");
        m.insert("setup_icons", "Selecciona el Set de Iconos:");
        m.insert("setup_theme", "Selecciona el Tema Inicial:");
        m.insert("repo_status", "Estado del Repositorio");
        m.insert("not_git_repo", "NO ES UN REPOSITORIO GIT");
        m.insert("suggest_cd", "Escribe /cd <ruta> para cambiar a un repositorio válido.");
        m.insert("status_ready", "Listo. Escribe / para ingresar un comando o /help para verlos.");
        m.insert("status_fetching", "Descargando actualizaciones del remoto...");
        m.insert("status_pulling", "Descargando y fusionando cambios remotos...");
        m.insert("status_pushing", "Subiendo commits al remoto...");
        m.insert("status_success", "¡Operación completada con éxito!");
        m.insert("status_not_git", "No se puede ejecutar: No estás en un repositorio Git.");
        m.insert("status_change_dir_err", "Error al cambiar de directorio: {}");
        m.insert("status_commit_success", "¡Commit creado con éxito!");
        m.insert("theme_title", "Seleccionar Tema");
        m.insert("footer_status", "Estado");
        m.insert("commands_list", "Comandos: /cd <ruta>, /fetch, /pull, /push, /commit <msg>, /themes, /quit");
        m.insert("welcome_message", "¡Bienvenido a Git Hero!");
        m.insert("help_header", "Comandos Disponibles");
        m.insert("setup_help", "Presiona [Arriba/Abajo] para elegir, [Enter] para confirmar.");
        m.insert("theme_help", "Presiona [Arriba/Abajo] para elegir, [Enter] para guardar, [Esc] para cancelar.");
        m.insert("status_stage_all_ok", "Todos los cambios preparados (staged).");
        m.insert("status_unstage_all_ok", "Todos los cambios quitados (unstaged).");
        m.insert("status_undo_commit", "Deshaciendo último commit (soft reset)...");
        m.insert("status_undo_commit_ok", "Último commit deshecho. Cambios conservados.");
        m.insert("status_remove_ok", "Repositorio Git eliminado. Directorio .git borrado.");
        m.insert("status_branch_list", "Lista de ramas cargada. Ver panel de diff.");
        m.insert("status_config_list", "Configuración cargada. Ver panel de diff.");
        m.insert("status_stash_ok", "Cambios guardados en stash.");
        m.insert("status_stash_pop_ok", "Stash recuperado. Cambios restaurados.");
        // ── Spanish versions of the new keys ──────────────────────
        m.insert("status_cd_ok", "Cambiado al directorio: {}");
        m.insert("status_cd_err", "Error al cambiar de directorio: {}");
        m.insert("status_remote_set", "Remoto origin → {}");
        m.insert("status_branch_deleted", "Rama '{}' eliminada.");
        m.insert("status_branch_switched", "Cambiado a la rama: {}");
        m.insert("status_branch_created", "Rama '{}' creada y activada.");
        m.insert("status_config_set_local", "Config local: {} = {}");
        m.insert("status_config_set_global", "Config global: {} = {}");
        m.insert("status_config_get_local", "Local: {} = {}");
        m.insert("status_config_get_global", "Global: {} = {}");
        m.insert("status_config_not_found", "Clave de config '{}' no encontrada.");
        m.insert("status_theme_changed", "Tema cambiado a: {}");
        m.insert("status_cmd_already_running", "Ya hay un comando en ejecución.");
        m.insert("status_remove_cancelled", "Eliminación del repositorio cancelada.");
        m.insert("status_err_commit_empty", "Error: mensaje de commit vacío.");
        m.insert("status_err_commit", "Error al hacer commit: {}");
        m.insert("status_err_stage", "Error al stagear todo: {}");
        m.insert("status_err_unstage", "Error al unstagear todo: {}");
        m.insert("status_err_undo_commit", "Error al deshacer commit: {}");
        m.insert("status_err_remove_repo", "Error al eliminar repo: {}");
        m.insert("status_err_set_remote", "Error al configurar remoto: {}");
        m.insert("status_err_branch_delete", "Error al eliminar rama: {}");
        m.insert("status_err_branch_create", "Error al crear rama: {}");
        m.insert("status_err_branch_switch", "Error al cambiar de rama: {}");
        m.insert("status_err_list_branches", "Error al listar ramas: {}");
        m.insert("status_err_list_config", "Error al listar config: {}");
        m.insert("status_err_config_get", "Error al leer config: {}");
        m.insert("status_err_config_set", "Error al configurar: {}");
        m.insert("status_err_config_set_global", "Error al configurar global: {}");
        m.insert("status_err_stash", "Error al guardar stash: {}");
        m.insert("status_err_stash_pop", "Error al recuperar stash: {}");
        m.insert("status_usage_remote", "Uso: /remote <url> - añade o actualiza el remoto origin");
        m.insert("status_usage_config", "Uso: /config <clave> [valor]");
        m.insert("status_usage_config_global", "Uso: /config-global <clave> [valor]");
        m.insert("ahead_label", "Adelante");
        m.insert("behind_label", "Detrás");
        m.insert("no_changes_to_commit", "Sin cambios para commit");
        m.insert("no_pending_changes", "Sin cambios pendientes para commit");
        m.insert("commit_message_label", "Mensaje del commit");
        m.insert("empty_message_cancelled", "Mensaje vacío, cancelado");
        m.insert("remote_has_changes", "El remoto tiene cambios");
        m.insert("do_pull", "¿Hacer pull?");
        m.insert("pull_completed", "Pull completado");
        m.insert("do_push", "¿Hacer push?");
        m.insert("push_completed", "Push completado");
        m.insert("local_commits_kept", "Commits locales guardados, sin push");
        m.insert("up_to_date_with_remote", "Estás al día con el remoto");
        m.insert("fetch_complete", "✓ Fetch completo.");
        // ── Update modal keys (ES) ─────────────────────────────
        m.insert("update_title", "🚀 Actualización Disponible");
        m.insert("update_new_version", "Una nueva versión de Git Hero está disponible: v{}");
        m.insert("update_current_version", "Versión actual: v{}");
        m.insert("update_yes", "Abrir página de descarga");
        m.insert("update_no", "Recordarme después");
        m.insert("update_skip", "No volver a mostrar esta versión");
        // ── /language command keys (ES) ────────────────────────
        m.insert("status_language_changed", "Idioma cambiado a: {}");
        m.insert("status_language_same", "El idioma ya es {}");
        m
    })
}

/// Look up a translation for `key` in the given language.
///
/// Returns `Cow::Borrowed` (no allocation) when the key is present in the
/// dictionary, and `Cow::Owned(key.to_string())` (one allocation) when the
/// key is missing — the key itself is shown to the user as a fallback so
/// typos are immediately visible.
pub fn translate(lang: &str, key: &str) -> Cow<'static, str> {
    let dict = if lang == "es" { es_dict() } else { en_dict() };
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
        let en = en_dict();
        let es = es_dict();
        for key in en.keys() {
            assert!(
                es.contains_key(key),
                "Spanish dict is missing key: {key}"
            );
        }
        for key in es.keys() {
            assert!(
                en.contains_key(key),
                "English dict is missing key: {key}"
            );
        }
    }
}
