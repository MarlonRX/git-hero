use std::collections::HashMap;

pub fn translate(lang: &str, key: &str) -> String {
    let mut en = HashMap::new();
    en.insert("setup_title", "Git Hero — First Time Setup");
    en.insert("setup_lang", "Select Language:");
    en.insert("setup_icons", "Select Icon Set (Nerd Fonts require compatible terminal font):");
    en.insert("setup_theme", "Select Initial Theme:");
    en.insert("repo_status", "Repository Status");
    en.insert("not_git_repo", "NOT A GIT REPOSITORY");
    en.insert("suggest_cd", "Type /cd <path> to switch to a valid repository.");
    en.insert("status_ready", "Ready. Type / to enter a command or /help to see commands.");
    en.insert("status_fetching", "Fetching remote updates...");
    en.insert("status_pulling", "Pulling remote updates...");
    en.insert("status_pushing", "Pushing commits to remote...");
    en.insert("status_success", "Operation completed successfully!");
    en.insert("status_not_git", "Cannot execute: Not inside a Git repository.");
    en.insert("status_change_dir_err", "Error changing directory: {}");
    en.insert("status_commit_success", "Commit created successfully!");
    en.insert("theme_title", "Select Theme");
    en.insert("footer_status", "Status");
    en.insert("commands_list", "Commands: /cd <path>, /fetch, /pull, /push, /commit <msg>, /themes, /quit");
    en.insert("welcome_message", "Welcome to Git Hero!");
    en.insert("help_header", "Available Commands");
    en.insert("setup_help", "Press [Up/Down] to select, [Enter] to confirm.");
    en.insert("theme_help", "Press [Up/Down] to select, [Enter] to save, [Esc] to cancel.");

    // ── New status messages ────────────────────────────────────
    en.insert("status_stage_all_ok", "Staged all changes.");
    en.insert("status_unstage_all_ok", "Unstaged all changes.");
    en.insert("status_undo_commit", "Undoing last commit (soft reset)...");
    en.insert("status_undo_commit_ok", "Last commit undone. Changes kept in working tree.");
    en.insert("status_remove_ok", "Git repository removed. .git directory deleted.");
    en.insert("status_branch_list", "Branch list loaded. See diff panel.");
    en.insert("status_config_list", "Config loaded. See diff panel.");
    en.insert("status_stash_ok", "Changes stashed.");
    en.insert("status_stash_pop_ok", "Stash popped. Changes restored.");

    let mut es = HashMap::new();
    es.insert("setup_title", "Git Hero — Configuración Inicial");
    es.insert("setup_lang", "Selecciona el Idioma:");
    es.insert("setup_icons", "Selecciona el Set de Iconos:");
    es.insert("setup_theme", "Selecciona el Tema Inicial:");
    es.insert("repo_status", "Estado del Repositorio");
    es.insert("not_git_repo", "NO ES UN REPOSITORIO GIT");
    es.insert("suggest_cd", "Escribe /cd <ruta> para cambiar a un repositorio válido.");
    es.insert("status_ready", "Listo. Escribe / para ingresar un comando o /help para verlos.");
    es.insert("status_fetching", "Descargando actualizaciones del remoto...");
    es.insert("status_pulling", "Descargando y fusionando cambios remotos...");
    es.insert("status_pushing", "Subiendo commits al remoto...");
    es.insert("status_success", "¡Operación completada con éxito!");
    es.insert("status_not_git", "No se puede ejecutar: No estás en un repositorio Git.");
    es.insert("status_change_dir_err", "Error al cambiar de directorio: {}");
    es.insert("status_commit_success", "¡Commit creado con éxito!");
    es.insert("theme_title", "Seleccionar Tema");
    es.insert("footer_status", "Estado");
    es.insert("commands_list", "Comandos: /cd <ruta>, /fetch, /pull, /push, /commit <msg>, /themes, /quit");
    es.insert("welcome_message", "¡Bienvenido a Git Hero!");
    es.insert("help_header", "Comandos Disponibles");
    es.insert("setup_help", "Presiona [Arriba/Abajo] para elegir, [Enter] para confirmar.");
    es.insert("theme_help", "Presiona [Arriba/Abajo] para elegir, [Enter] para guardar, [Esc] para cancelar.");

    // ── New status messages (Spanish) ───────────────────────────
    es.insert("status_stage_all_ok", "Todos los cambios preparados (staged).");
    es.insert("status_unstage_all_ok", "Todos los cambios quitados (unstaged).");
    es.insert("status_undo_commit", "Deshaciendo \u{00FA}ltimo commit (soft reset)...");
    es.insert("status_undo_commit_ok", "\u{00FA}ltimo commit deshecho. Cambios conservados.");
    es.insert("status_remove_ok", "Repositorio Git eliminado. Directorio .git borrado.");
    es.insert("status_branch_list", "Lista de ramas cargada. Ver panel de diff.");
    es.insert("status_config_list", "Configuraci\u{00F3}n cargada. Ver panel de diff.");
    es.insert("status_stash_ok", "Cambios guardados en stash.");
    es.insert("status_stash_pop_ok", "Stash recuperado. Cambios restaurados.");

    let dict = match lang {
        "es" => es,
        _ => en,
    };

    dict.get(key).unwrap_or(&key).to_string()
}
