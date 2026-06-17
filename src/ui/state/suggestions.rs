use std::fs;
use std::path::Path;
use crate::ui::state::AppState;

impl AppState {
    pub fn update_suggestions(&mut self) {
        let val = &self.input_value;
        if val.starts_with("/cd ") {
            self.suggestions = get_directory_suggestions(val);
        } else if val.starts_with('/') {
            self.suggestions = get_command_suggestions(val);
        } else {
            self.suggestions.clear();
        }
        if !self.suggestions.is_empty() && self.active_sug >= self.suggestions.len() {
            self.active_sug = 0;
        }
    }
}

pub fn expand_path(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let home_str = home.to_string_lossy().into_owned();
            return path.replacen('~', &home_str, 1);
        }
    }
    path.to_string()
}

pub fn get_directory_suggestions(input: &str) -> Vec<String> {
    if !input.starts_with("/cd ") {
        return Vec::new();
    }
    let path_arg = &input[4..];
    let resolved_path = expand_path(path_arg);

    let (search_dir, prefix) = if path_arg.is_empty() {
        (".", "")
    } else if path_arg == "~" {
        (&resolved_path as &str, "")
    } else if path_arg.ends_with('/') || path_arg.ends_with('\\') {
        (&resolved_path as &str, "")
    } else {
        let path = Path::new(&resolved_path);
        let parent = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        (parent, file_name)
    };

    let mut suggestions = Vec::new();
    if let Ok(entries) = fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if prefix.is_empty()
                        || name.to_lowercase().starts_with(&prefix.to_lowercase())
                    {
                        let mut base_path = path_arg.to_string();
                        if !prefix.is_empty() {
                            base_path =
                                path_arg[..path_arg.len() - prefix.len()].to_string();
                        }
                        if !base_path.is_empty()
                            && !base_path.ends_with('/')
                            && !base_path.ends_with('\\')
                        {
                            base_path.push('/');
                        }
                        suggestions.push(format!("/cd {}{}/", base_path, name));
                    }
                }
            }
        }
    }
    suggestions.truncate(5);
    suggestions
}

pub fn get_command_suggestions(input: &str) -> Vec<String> {
    let commands = vec![
        "/fetch".to_string(),
        "/pull".to_string(),
        "/push".to_string(),
        "/commit ".to_string(),
        "/stage-all".to_string(),
        "/unstage-all".to_string(),
        "/undo-commit".to_string(),
        "/remove-repo".to_string(),
        "/remote ".to_string(),
        "/branch ".to_string(),
        "/branches".to_string(),
        "/switch ".to_string(),
        "/config ".to_string(),
        "/config-global ".to_string(),
        "/stash".to_string(),
        "/stash-pop".to_string(),
        "/cd ".to_string(),
        "/themes".to_string(),
        "/help".to_string(),
        "/docs".to_string(),
        "/quit".to_string(),
    ];
    if input.is_empty() || input == "/" {
        return commands;
    }
    commands
        .into_iter()
        .filter(|c| c.starts_with(input))
        .collect()
}
