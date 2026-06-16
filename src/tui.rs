use std::fs;
use std::io::{self, Stdout};
use std::path::Path;
use std::time::Duration;

use crossterm::{
	event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
	Frame, Terminal,
};

use crate::config::{load_config, save_config, Config};
use crate::git;
use crate::i18n::translate;
use crate::theme::{get_theme_by_name, get_themes, Theme};

#[derive(Clone)]
pub struct GitFile {
	pub path: String,
	pub staged: bool,
	pub status: String,
	pub info: String,
}

#[derive(Clone)]
pub struct GitCommit {
	pub hash: String,
	pub date: String,
	pub author: String,
	pub subject: String,
}

pub struct AppState {
	// Git State
	pub cwd: String,
	pub is_git_repo: bool,
	pub branch: String,
	pub remote: String,
	pub behind: i32,
	pub ahead: i32,

	// Config State
	pub theme: Theme,
	pub language: String,
	pub nerd_font: bool,
	pub status_message: String,
	pub fetching: bool,

	// Interactive lists
	pub files: Vec<GitFile>,
	pub selected_file_idx: usize,
	pub commits: Vec<GitCommit>,
	pub selected_commit_idx: usize,
	pub focus_pane: String, // "files", "commits", "setup", "init", "themes", "help"
	pub active_diff: String,
	pub diff_scroll_offset: usize,

	// Command Input bar
	pub input_value: String,
	pub input_cursor_pos: usize,
	pub show_input: bool,
	pub suggestions: Vec<String>,
	pub active_sug: usize,

	// Modals
	pub show_theme_modal: bool,
	pub show_help_modal: bool,
	pub theme_cursor: usize,
	pub saved_theme: String,

	// Setup Wizard
	pub setup_step: usize,
	pub setup_cursor: usize,

	// Git Init Wizard
	pub init_wizard_active: bool,
	pub init_wizard_step: usize,
	pub init_cursor: usize,
	pub init_branch_name: String,
	pub init_remote_url: String,
}

impl AppState {
	pub fn new() -> Self {
		let config = load_config().unwrap_or(Config {
			language: "en".to_string(),
			nerd_font: false,
			theme: "Tokyo Night".to_string(),
		});

		let theme = get_theme_by_name(&config.theme);
		let cwd = std::env::current_dir()
			.map(|p| p.to_string_lossy().into_owned())
			.unwrap_or_else(|_| ".".to_string());

		let mut state = Self {
			cwd,
			is_git_repo: false,
			branch: String::new(),
			remote: String::new(),
			behind: 0,
			ahead: 0,
			theme,
			language: config.language.clone(),
			nerd_font: config.nerd_font,
			status_message: "Ready. Press ? for help.".to_string(),
			fetching: false,
			files: Vec::new(),
			selected_file_idx: 0,
			commits: Vec::new(),
			selected_commit_idx: 0,
			focus_pane: "files".to_string(),
			active_diff: String::new(),
			diff_scroll_offset: 0,
			input_value: String::new(),
			input_cursor_pos: 0,
			show_input: false,
			suggestions: Vec::new(),
			active_sug: 0,
			show_theme_modal: false,
			show_help_modal: false,
			theme_cursor: 0,
			saved_theme: config.theme,
			setup_step: 0,
			setup_cursor: 0,
			init_wizard_active: false,
			init_wizard_step: 0,
			init_cursor: 0,
			init_branch_name: String::new(),
			init_remote_url: String::new(),
			sprite_frames: vec!["🤖", "⡿", "⣟", "⣯", "⣿"],
			sprite_index: 0,
		};

		if config.language.is_empty() {
			state.setup_step = 1;
			state.status_message = "Welcome! Please configure Git Hero.".to_string();
		} else {
			state.refresh_git_status();
		}

		state
	}

	pub fn refresh_git_status(&mut self) {
		self.is_git_repo = git::is_inside_work_tree();
		if self.is_git_repo {
			self.branch = git::get_current_branch().unwrap_or_else(|_| "".to_string());
			self.remote = git::get_remote(&self.branch);
			self.behind = git::get_commits_behind(&self.remote, &self.branch);
			self.ahead = git::get_commits_ahead(&self.remote, &self.branch);

			self.files = self.get_changed_files();
			self.commits = self.get_recent_commits();

			if !self.files.is_empty() && self.selected_file_idx >= self.files.len() {
				self.selected_file_idx = 0;
			}
			if !self.commits.is_empty() && self.selected_commit_idx >= self.commits.len() {
				self.selected_commit_idx = 0;
			}

			self.update_diff_content();
		} else {
			self.branch.clear();
			self.remote.clear();
			self.behind = 0;
			self.ahead = 0;
			self.files.clear();
			self.commits.clear();
			self.active_diff.clear();
			self.status_message = "Warning: Not a Git repository.".to_string();
		}
	}

	fn get_changed_files(&self) -> Vec<GitFile> {
		let mut files = Vec::new();
		if let Ok(out) = git::run_git(&["status", "--porcelain"]) {
			if !out.is_empty() {
				for line in out.split('\n') {
					if line.len() < 4 {
						continue;
					}
					let status = line[0..2].to_string();
					let mut path = line[3..].to_string();
					if path.starts_with('"') && path.ends_with('"') {
						path = path[1..path.len() - 1].to_string();
					}
					let staged = status.chars().next().unwrap_or(' ') != ' '
						&& status.chars().next().unwrap_or(' ') != '?';
					let info = match status.as_str() {
						"M " | " M" | "MM" => "modified",
						"A " => "added",
						"D " | " D" => "deleted",
						"??" => "untracked",
						_ => "changed",
					};
					files.push(GitFile {
						path,
						staged,
						status: status.trim().to_string(),
						info: info.to_string(),
					});
				}
			}
		}
		files
	}

	fn get_recent_commits(&self) -> Vec<GitCommit> {
		let mut commits = Vec::new();
		if let Ok(out) = git::run_git(&["log", "-n", "15", "--pretty=format:%h|%ar|%an|%s"]) {
			if !out.is_empty() {
				for line in out.split('\n') {
					let parts: Vec<&str> = line.split('|').collect();
					if parts.len() >= 4 {
						commits.push(GitCommit {
							hash: parts[0].to_string(),
							date: parts[1].to_string(),
							author: parts[2].to_string(),
							subject: parts[3].to_string(),
						});
					}
				}
			}
		}
		commits
	}

	fn update_diff_content(&mut self) {
		if self.focus_pane == "commits" && !self.commits.is_empty() && self.selected_commit_idx < self.commits.len() {
			let hash = &self.commits[self.selected_commit_idx].hash;
			self.active_diff = match git::run_git(&["show", "--stat", "--patch", hash]) {
				Ok(out) => out,
				Err(e) => format!("Error showing commit: {}", e),
			};
		} else if !self.files.is_empty() && self.selected_file_idx < self.files.len() {
			let file = &self.files[self.selected_file_idx];
			if file.status == "??" {
				self.active_diff = match fs::read_to_string(&file.path) {
					Ok(content) => {
						let lines: Vec<&str> = content.split('\n').collect();
						if lines.len() > 100 {
							let mut truncated = lines[..100].join("\n");
							truncated.push_str("\n... (truncated)");
							truncated
						} else {
							content
						}
					}
					Err(e) => format!("Error reading untracked file: {}", e),
				};
			} else {
				let args = if file.staged {
					vec!["diff", "--cached", "--", &file.path]
				} else {
					vec!["diff", "--", &file.path]
				};
				self.active_diff = match git::run_git(&args) {
					Ok(out) => {
						if out.is_empty() {
							"No changes.".to_string()
						} else {
							out
						}
					}
					Err(e) => format!("Error loading diff: {}", e),
				};
			}
		} else {
			self.active_diff = "Working directory clean.".to_string();
		}
	}

	fn toggle_stage_file(&mut self, idx: usize) {
		if idx >= self.files.len() {
			return;
		}
		let file = &self.files[idx];
		let args = if file.staged {
			vec!["restore", "--staged", "--", &file.path]
		} else {
			vec!["add", "--", &file.path]
		};
		let _ = git::run_git(&args);
		self.refresh_git_status();
	}

	pub fn get_icon_str(&self, key: &str) -> &'static str {
		if self.nerd_font {
			match key {
				"branch" => "",
				"dir" => "",
				"fetch" => "",
				"commit" => "",
				"mod" => "",
				"add" => "",
				"del" => "",
				"untracked" => "",
				"ok" => "✔",
				"warn" => "⚠",
				_ => "",
			}
		} else {
			match key {
				"branch" => "*",
				"dir" => "/",
				"fetch" => "~",
				"commit" => "#",
				"mod" => "~",
				"add" => "+",
				"del" => "-",
				"untracked" => "?",
				"ok" => "O",
				"warn" => "!",
				_ => "",
			}
		}
	}

	fn execute_command(&mut self, input: &str) {
		if input.starts_with("/cd ") {
			let path = &input[4..];
			let resolved = expand_path(path);
			if std::env::set_current_dir(resolved).is_ok() {
				if let Ok(new_cwd) = std::env::current_dir() {
					self.cwd = new_cwd.to_string_lossy().into_owned();
					self.selected_file_idx = 0;
					self.selected_commit_idx = 0;
					self.diff_scroll_offset = 0;
					self.refresh_git_status();
				}
			} else {
				self.status_message = format!("Error changing directory to: {}", path);
			}
			return;
		}

		if input == "/fetch" {
			if !self.is_git_repo {
				self.status_message = translate(&self.language, "status_not_git");
				return;
			}
			self.fetching = true;
			self.status_message = translate(&self.language, "status_fetching");
			let remote = self.remote.clone();
			let branch = self.branch.clone();
			let _ = git::fetch_remote(&remote, &branch);
			self.fetching = false;
			self.refresh_git_status();
			return;
		}

		if input == "/pull" {
			if !self.is_git_repo {
				self.status_message = translate(&self.language, "status_not_git");
				return;
			}
			self.status_message = translate(&self.language, "status_pulling");
			let remote = self.remote.clone();
			let branch = self.branch.clone();
			let _ = git::git_pull(&remote, &branch);
			self.refresh_git_status();
			return;
		}

		if input == "/push" {
			if !self.is_git_repo {
				self.status_message = translate(&self.language, "status_not_git");
				return;
			}
			self.status_message = translate(&self.language, "status_pushing");
			let remote = self.remote.clone();
			let branch = self.branch.clone();
			let _ = git::git_push(&remote, &branch);
			self.refresh_git_status();
			return;
		}

		if input.starts_with("/commit ") {
			if !self.is_git_repo {
				self.status_message = translate(&self.language, "status_not_git");
				return;
			}
			let msg = &input[8..];
			if msg.is_empty() {
				self.status_message = "Error: commit message empty.".to_string();
				return;
			}

			let has_staged = self.files.iter().any(|f| f.staged);
			if !has_staged {
				let _ = git::git_add_all();
			}

			if let Err(e) = git::git_commit(msg) {
				self.status_message = format!("Error committing: {}", e);
			} else {
				self.selected_file_idx = 0;
				self.selected_commit_idx = 0;
				self.diff_scroll_offset = 0;
				self.refresh_git_status();
				self.status_message = translate(&self.language, "status_commit_success");
			}
			return;
		}

		if input == "/themes" {
			self.show_theme_modal = true;
			self.saved_theme = self.theme.name.to_string();
			self.theme_cursor = 0;
			let themes = get_themes();
			for (i, t) in themes.iter().enumerate() {
				if t.name == self.theme.name {
					self.theme_cursor = i;
					break;
				}
			}
			return;
		}

		if input == "/help" {
			self.show_help_modal = true;
			return;
		}

		self.status_message = format!("Unknown command: {}. Type /help.", input);
	}

	fn update_suggestions(&mut self) {
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

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	let mut state = AppState::new();

	loop {
		terminal.draw(|f| draw_ui(f, &mut state))?;

		if event::poll(Duration::from_millis(100))? {
			if let Event::Key(key) = event::read()? {
				if key.kind == crossterm::event::KeyEventKind::Press {
					// Global quit
					if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
						break;
					}

					if !handle_key_event(key.code, &mut state) {
						break;
					}
				}
			} else if let Event::Mouse(mouse) = event::read()? {
				if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
					handle_mouse_click(mouse.column, mouse.row, &mut state, &terminal);
				}
			}
		}
	}

	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		crossterm::event::DisableMouseCapture
	)?;
	terminal.show_cursor()?;
	Ok(())
}

// ── Rendering Layout ─────────────────────────────────────────────────────

fn draw_ui(f: &mut Frame, s: &mut AppState) {
	let area = f.area();

	// Setup Step
	if s.setup_step > 0 {
		draw_setup_wizard(f, s);
		return;
	}

	// 1. FONDO COMPLETO DEL TEMA - Aplica el color de fondo en TODA el área
	// Esto evita que se vea el fondo negro de la terminal
	let full_bg = Style::default().bg(s.theme.background);
	f.render_widget(
		Paragraph::new("").style(full_bg),
		area
	);

	// 2. LAYOUT CENTRADO AL 80%
	// Calculamos el tamaño para que la UI ocupe el 80% del espacio disponible
	// y quede centrada tanto vertical como horizontalmente
	let target_w = (area.width as f32 * 0.80) as u16;
	let target_h = (area.height as f32 * 0.85) as u16;

	let outer_rect = Rect {
		x: area.x + (area.width.saturating_sub(target_w)) / 2,
		y: area.y + (area.height.saturating_sub(target_h)) / 2,
		width: target_w.max(40),
		height: target_h.max(10),
	};

	if outer_rect.width < 20 || outer_rect.height < 8 {
		return;
	}

	// 3. BORDE EXTERIOR CON BLOQUES SÓLIDOS (compatible con Warp)
	let border_style = Style::default().bg(s.theme.border).fg(s.theme.border);

	// Borde superior (fila sólida)
	f.render_widget(
		Paragraph::new("█".repeat(outer_rect.width as usize)).style(border_style),
		Rect { x: outer_rect.x, y: outer_rect.y, width: outer_rect.width, height: 1 }
	);
	// Borde inferior
	f.render_widget(
		Paragraph::new("█".repeat(outer_rect.width as usize)).style(border_style),
		Rect { x: outer_rect.x, y: outer_rect.y + outer_rect.height - 1, width: outer_rect.width, height: 1 }
	);
	// Bordes laterales (columnas sólidas)
	for row in 1..outer_rect.height.saturating_sub(1) {
		let y_pos = outer_rect.y + row;
		f.render_widget(
			Paragraph::new("█").style(border_style),
			Rect { x: outer_rect.x, y: y_pos, width: 1, height: 1 }
		);
		f.render_widget(
			Paragraph::new("█").style(border_style),
			Rect { x: outer_rect.x + outer_rect.width - 1, y: y_pos, width: 1, height: 1 }
		);
	}

	// Get the inner area (inside the border) for content positioning
	let inner_rect = Rect {
		x: outer_rect.x + 1,
		y: outer_rect.y + 1,
		width: outer_rect.width.saturating_sub(2),
		height: outer_rect.height.saturating_sub(2),
	};

	// 4. HEADER - Título con fondo del tema primario (corta el borde superior)
	let header_text = format!(" Git Hero {} ", s.get_icon_str("commit"));
	let header_style = Style::default().fg(s.theme.background).bg(s.theme.primary).add_modifier(Modifier::BOLD);
	f.render_widget(
		Paragraph::new(header_text).style(header_style),
		Rect { x: outer_rect.x + 2, y: outer_rect.y, width: 25, height: 1 }
	);

	// Theme indicator en el header
	let theme_name = format!(" {} ", s.theme.name);
	f.render_widget(
		Paragraph::new(theme_name).style(Style::default().fg(s.theme.background).bg(s.theme.accent).add_modifier(Modifier::BOLD)),
		Rect { x: outer_rect.x + outer_rect.width.saturating_sub(22), y: outer_rect.y, width: 20, height: 1 }
	);

	// 5. LAYOUT INTERIOR
	// Reservamos espacio para el footer (2 líneas) y la barra de comandos
	let footer_h: u16 = 2;
	let body_height = inner_rect.height.saturating_sub(footer_h);
	let body_rect = Rect {
		x: inner_rect.x,
		y: inner_rect.y,
		width: inner_rect.width,
		height: body_height,
	};

	// Footer area
	let footer_rect = Rect {
		x: inner_rect.x,
		y: inner_rect.y + body_height,
		width: inner_rect.width,
		height: footer_h,
	};

	// Renderizar el contenido principal
	if !s.is_git_repo && !s.init_wizard_active {
		draw_no_repo_panel(f, s, body_rect);
	} else if s.init_wizard_active {
		draw_init_wizard(f, s, body_rect);
	} else {
		draw_dashboard(f, s, body_rect);
	}

	// 6. FOOTER - Barra de estado con mejor diseño
	// Línea separadora del footer
	let footer_separator_style = Style::default().bg(s.theme.border).fg(s.theme.border);
	f.render_widget(
		Paragraph::new("█".repeat(inner_rect.width as usize)).style(footer_separator_style),
		Rect { x: footer_rect.x, y: footer_rect.y, width: footer_rect.width, height: 1 }
	);

	// Status message con icono
	let status_icon = if s.fetching { format!("{} ", s.get_icon_str("fetch")) } else { " ".to_string() };
	let status_str = format!(" {}{}", status_icon, s.status_message);
	let status_style = if s.fetching {
		Style::default().fg(s.theme.warning).bg(s.theme.background)
	} else {
		Style::default().fg(s.theme.success).bg(s.theme.background)
	};
	f.render_widget(
		Paragraph::new(status_str).style(status_style),
		Rect { x: footer_rect.x + 1, y: footer_rect.y + 1, width: footer_rect.width / 2, height: 1 }
	);

	// Keyboard Legend - alineado a la derecha
	let mut legend = "Spc:Stage Tab:Focus c:Commit p:Push f:Fetch l:Pull t:Theme ?:Help q:Quit";
	if s.language == "es" {
		legend = "Esp:Stage Tab:Foco c:Commit p:Push f:Fetch l:Pull t:Tema ?:Ayuda q:Salir";
	}
	let legend_len = legend.chars().count() as u16;
	f.render_widget(
		Paragraph::new(legend).style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
		Rect {
			x: footer_rect.x + footer_rect.width.saturating_sub(legend_len + 1),
			y: footer_rect.y + 1,
			width: legend_len,
			height: 1,
		}
	);

	// 7. BARRA DE COMANDOS DEDICADA (overlay en el borde inferior)
	if s.show_input && !s.show_theme_modal && !s.show_help_modal {
		let input_y = outer_rect.y + outer_rect.height - 1;
		let input_area = Rect {
			x: outer_rect.x + 1,
			y: input_y,
			width: outer_rect.width.saturating_sub(2),
			height: 1
		};

		// Fondo del input con color primario para destacar
		let input_bg = Style::default().bg(s.theme.primary);
		f.render_widget(
			Paragraph::new(" ".repeat(input_area.width as usize)).style(input_bg),
			input_area
		);

		// Prompt "❯" para indicar que es input
		let prompt_style = Style::default().fg(s.theme.accent).bg(s.theme.primary).add_modifier(Modifier::BOLD);
		f.render_widget(
			Paragraph::new(" \u{276F} ").style(prompt_style),
			Rect { x: input_area.x, y: input_y, width: 3, height: 1 }
		);

		// Texto del input
		let display_val = if s.input_value.is_empty() {
			Span::styled("Type a command...", Style::default().fg(s.theme.dimmed).bg(s.theme.primary))
		} else {
			Span::styled(&s.input_value, Style::default().fg(s.theme.background).bg(s.theme.primary).add_modifier(Modifier::BOLD))
		};
		let text_area = Rect { x: input_area.x + 3, y: input_y, width: input_area.width.saturating_sub(4), height: 1 };
		f.render_widget(Paragraph::new(Line::from(vec![display_val])), text_area);

		// Cursor block
		let cursor_x = text_area.x + s.input_cursor_pos as u16;
		if cursor_x < text_area.x + text_area.width {
			f.render_widget(
				Paragraph::new(" ").style(Style::default().bg(s.theme.accent)),
				Rect { x: cursor_x, y: input_y, width: 1, height: 1 }
			);
		}

		// Suggestions box above input bar
		if !s.suggestions.is_empty() {
			let sug_box_h = s.suggestions.len() as u16;
			let sug_area = Rect {
				x: outer_rect.x + 1,
				y: input_y.saturating_sub(sug_box_h + 1),
				width: 40,
				height: sug_box_h,
			};
			f.render_widget(Clear, sug_area);

			let items: Vec<ListItem> = s.suggestions.iter().enumerate().map(|(i, sug)| {
				let style = if i == s.active_sug {
					Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
				} else {
					Style::default().fg(s.theme.foreground).bg(s.theme.background)
				};
				let prefix = if i == s.active_sug { " > " } else { "   " };
				ListItem::new(format!("{}{}", prefix, sug)).style(style)
			}).collect();

			let sug_list = List::new(items)
				.block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(s.theme.dimmed)));
			f.render_widget(sug_list, Rect { x: sug_area.x, y: sug_area.y.saturating_sub(1), width: sug_area.width, height: sug_area.height + 2 });
		}
	}

	// Floating Modals
	if s.show_theme_modal {
		draw_theme_modal(f, s);
	} else if s.show_help_modal {
		draw_help_modal(f, s);
	}
}

fn draw_no_repo_panel(f: &mut Frame, s: &mut AppState, body_rect: Rect) {
	// Aplicar fondo del tema en toda el área del panel
	f.render_widget(
		Paragraph::new("").style(Style::default().bg(s.theme.background)),
		body_rect
	);

	let panel_area = Rect {
		x: body_rect.x + 1,
		y: body_rect.y + 1,
		width: body_rect.width.saturating_sub(2),
		height: body_rect.height.saturating_sub(2),
	};

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(s.theme.dimmed))
		.title(" No Git Repository Detected ")
		.title_style(Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD));
	f.render_widget(block, panel_area);

	// Fondo interior del panel
	f.render_widget(
		Paragraph::new("").style(Style::default().bg(s.theme.background)),
		Rect { x: panel_area.x + 1, y: panel_area.y + 1, width: panel_area.width.saturating_sub(2), height: panel_area.height.saturating_sub(2) }
	);

	let content_y = panel_area.y + 3;
	f.render_widget(
		Paragraph::new("This directory is not inside a Git repository.")
			.alignment(ratatui::layout::Alignment::Center)
			.style(Style::default().fg(s.theme.warning).bg(s.theme.background)),
		Rect { x: panel_area.x + 2, y: content_y, width: panel_area.width - 4, height: 1 }
	);

	f.render_widget(
		Paragraph::new(format!("Current path: {}", s.cwd))
			.alignment(ratatui::layout::Alignment::Center)
			.style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
		Rect { x: panel_area.x + 2, y: content_y + 1, width: panel_area.width - 4, height: 1 }
	);

	f.render_widget(
		Paragraph::new("Options:")
			.alignment(ratatui::layout::Alignment::Center)
			.style(Style::default().fg(s.theme.foreground).bg(s.theme.background).add_modifier(Modifier::BOLD)),
		Rect { x: panel_area.x + 2, y: content_y + 4, width: panel_area.width - 4, height: 1 }
	);

	let opt1 = "[1] Initialize Git repository here";
	let opt2 = "[2] Change Directory (Type /cd <path>)";

	let opt1_style = if s.init_cursor == 0 {
		Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
	} else {
		Style::default().fg(s.theme.foreground).bg(s.theme.background)
	};
	let opt2_style = if s.init_cursor == 1 {
		Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
	} else {
		Style::default().fg(s.theme.foreground).bg(s.theme.background)
	};

	f.render_widget(
		Paragraph::new(opt1).alignment(ratatui::layout::Alignment::Center).style(opt1_style),
		Rect { x: panel_area.x + 2, y: content_y + 6, width: panel_area.width - 4, height: 1 }
	);
	f.render_widget(
		Paragraph::new(opt2).alignment(ratatui::layout::Alignment::Center).style(opt2_style),
		Rect { x: panel_area.x + 2, y: content_y + 8, width: panel_area.width - 4, height: 1 }
	);

	f.render_widget(
		Paragraph::new("Use arrow keys/Enter or click with your mouse to select.")
			.alignment(ratatui::layout::Alignment::Center)
			.style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
		Rect { x: panel_area.x + 2, y: content_y + 11, width: panel_area.width - 4, height: 1 }
	);
}

fn draw_init_wizard(f: &mut Frame, s: &mut AppState, body_rect: Rect) {
	// Aplicar fondo del tema
	f.render_widget(
		Paragraph::new("").style(Style::default().bg(s.theme.background)),
		body_rect
	);

	let panel_area = Rect {
		x: body_rect.x + 1,
		y: body_rect.y + 1,
		width: body_rect.width.saturating_sub(2),
		height: body_rect.height.saturating_sub(2),
	};

	let mut title = " Git Initialization - Step 1/3 (Branch Name) ";
	let mut lines = Vec::new();

	match s.init_wizard_step {
		1 => {
			lines.push("Choose default branch name:".to_string());
			lines.push("".to_string());
			let opts = vec!["main", "master", "Custom (type name below)"];
			for (i, o) in opts.iter().enumerate() {
				if i == s.init_cursor {
					lines.push(format!("   > [{}] {}", i + 1, o));
				} else {
					lines.push(format!("     [{}] {}", i + 1, o));
				}
			}
			if s.init_cursor == 2 {
				lines.push("".to_string());
				lines.push(format!("   Branch name: {}", s.input_value));
			}
		}
		2 => {
			title = " Git Initialization - Step 2/3 (Remote URL) ";
			lines.push("Enter remote repository URL (optional):".to_string());
			lines.push("".to_string());
			lines.push(format!("   URL: {}", s.input_value));
			lines.push("".to_string());
			lines.push("Press [Enter] to continue or leave empty to skip.".to_string());
		}
		3 => {
			title = " Git Initialization - Step 3/3 (Confirm) ";
			let remote_text = if s.init_remote_url.is_empty() { "None" } else { &s.init_remote_url };
			lines.push("Review initialization details:".to_string());
			lines.push("".to_string());
			lines.push(format!("   Path:   {}", s.cwd));
			lines.push(format!("   Branch: {}", s.init_branch_name));
			lines.push(format!("   Remote: {}", remote_text));
			lines.push("".to_string());
			if s.init_cursor == 0 {
				lines.push("   > [1] Initialize Repository".to_string());
				lines.push("     [2] Cancel".to_string());
			} else {
				lines.push("     [1] Initialize Repository".to_string());
				lines.push("   > [2] Cancel".to_string());
			}
		}
		_ => {}
	}

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(s.theme.dimmed))
		.title(title)
		.title_style(Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD));
	f.render_widget(block, panel_area);

	// Fondo interior del panel
	f.render_widget(
		Paragraph::new("").style(Style::default().bg(s.theme.background)),
		Rect { x: panel_area.x + 1, y: panel_area.y + 1, width: panel_area.width.saturating_sub(2), height: panel_area.height.saturating_sub(2) }
	);

	let content_y = panel_area.y + 2;
	let content_str = lines.join("\n");
	f.render_widget(
		Paragraph::new(content_str).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
		Rect { x: panel_area.x + 2, y: content_y, width: panel_area.width - 4, height: panel_area.height - 4 }
	);
}

fn draw_dashboard(
	f: &mut Frame,
	s: &mut AppState,
	body_rect: Rect,
) {
	// ── Tab Header Info ──
	let branch_icon = s.get_icon_str("branch");
	let branch_str = format!(" {} {} ", branch_icon, s.branch);
	f.render_widget(
		Paragraph::new(branch_str).style(Style::default().fg(s.theme.primary).bg(s.theme.background).add_modifier(Modifier::BOLD)),
		Rect { x: body_rect.x, y: body_rect.y, width: 20, height: 1 }
	);

	let dir_icon = s.get_icon_str("dir");
	let mut dir_path = s.cwd.clone();
	if dir_path.len() > 30 {
		dir_path = format!("...{}", &dir_path[dir_path.len() - 27..]);
	}
	let dir_str = format!(" {} {} ", dir_icon, dir_path);
	f.render_widget(
		Paragraph::new(dir_str).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
		Rect { x: body_rect.x + 20, y: body_rect.y, width: 35, height: 1 }
	);

	let remote_str = format!(" {} {} ", s.get_icon_str("fetch"), s.remote);
	f.render_widget(
		Paragraph::new(remote_str).style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
		Rect { x: body_rect.x + 55, y: body_rect.y, width: 25, height: 1 }
	);

	if body_rect.width >= 50 {
		// Use ratatui Layout for consistent, compatible rendering
		let sidebar_w = (body_rect.width / 4).max(20);
		let right_w = body_rect.width.saturating_sub(sidebar_w);
		let header_h: u16 = 2;

		// Main horizontal split: sidebar | right panel
		let main_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Length(sidebar_w), Constraint::Min(20)])
			.split(Rect { x: body_rect.x, y: body_rect.y + header_h, width: body_rect.width, height: body_rect.height.saturating_sub(header_h) });

		let sidebar_area = main_chunks[0];
		let right_area = main_chunks[1];

		// ─── SIDEBAR ───
		// Vertical split in sidebar: info (fixed) | files (remaining)
		let sidebar_chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Length(4), Constraint::Min(3)])
			.split(sidebar_area);

		let info_block = Block::default()
			.borders(Borders::RIGHT)
			.border_style(Style::default().fg(s.theme.border));
		f.render_widget(info_block.clone(), sidebar_chunks[0]);
		let info_inner = info_block.inner(sidebar_chunks[0]);

		let behind_text = format!(" Behind: {} commits", s.behind);
		let ahead_text = format!(" Ahead:  {} commits", s.ahead);
		let behind_style = if s.behind > 0 { Style::default().fg(s.theme.warning) } else { Style::default().fg(s.theme.dimmed) };
		let ahead_style = if s.ahead > 0 { Style::default().fg(s.theme.success) } else { Style::default().fg(s.theme.dimmed) };

		let details = Paragraph::new(vec![
			Line::from(Span::styled("STATUS INFO", Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD))),
			Line::from(Span::styled(behind_text, behind_style)),
			Line::from(Span::styled(ahead_text, ahead_style)),
		]).style(Style::default().bg(s.theme.background));
		f.render_widget(details, info_inner);

		let files_block = Block::default()
			.borders(Borders::RIGHT | Borders::TOP)
			.border_style(Style::default().fg(s.theme.border))
			.title(format!(" Changed Files ({}) ", s.files.len()))
			.title_style(if s.focus_pane == "files" {
				Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
			} else {
				Style::default().fg(s.theme.foreground).add_modifier(Modifier::BOLD)
			});
		f.render_widget(files_block.clone(), sidebar_chunks[1]);
		let files_inner = files_block.inner(sidebar_chunks[1]);

		if s.files.is_empty() {
			let clean = if s.language == "es" { "Directorio limpio." } else { "Working directory clean." };
			f.render_widget(
				Paragraph::new(clean).style(Style::default().fg(s.theme.success).bg(s.theme.background)),
				files_inner
			);
		} else {
			let items: Vec<ListItem> = s.files.iter().enumerate().map(|(i, f)| {
				let file_style = if f.staged {
					Style::default().fg(s.theme.success)
				} else if f.status == "??" {
					Style::default().fg(s.theme.dimmed)
				} else {
					Style::default().fg(s.theme.warning)
				};

				let cb = if f.staged { "[✔] " } else { "[ ] " };
				let icon = match f.status.as_str() {
					"A" => s.get_icon_str("add"),
					"D" => s.get_icon_str("del"),
					"??" => s.get_icon_str("untracked"),
					_ => s.get_icon_str("mod"),
				};

				let prefix = if i == s.selected_file_idx && s.focus_pane == "files" { "> " } else { "  " };
				let line = format!("{}{}{} {}", prefix, cb, icon, f.path);
				
				let item_style = if i == s.selected_file_idx && s.focus_pane == "files" {
					file_style.bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
				} else {
					file_style.bg(s.theme.background)
				};

				ListItem::new(line).style(item_style)
			}).collect();

			let list = List::new(items).style(Style::default().bg(s.theme.background));
			f.render_widget(list, files_inner);
		}

		// ─── RIGHT PANEL ───
		// Vertical split: diff (top) | commits (bottom)
		let right_chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
			.split(right_area);

		// Top: Diff viewer
		let mut diff_title = " Active Changes / Diff ".to_string();
		if s.focus_pane == "commits" && !s.commits.is_empty() && s.selected_commit_idx < s.commits.len() {
			diff_title = format!(" Commit detail: {} ", s.commits[s.selected_commit_idx].hash);
		} else if !s.files.is_empty() && s.selected_file_idx < s.files.len() {
			diff_title = format!(" Diff: {} ", s.files[s.selected_file_idx].path);
		}

		let diff_block = Block::default()
			.borders(Borders::BOTTOM)
			.border_style(Style::default().fg(s.theme.border))
			.title(diff_title)
			.title_style(Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD));
		f.render_widget(diff_block.clone(), right_chunks[0]);
		let diff_inner = diff_block.inner(right_chunks[0]);

		let lines: Vec<Line> = s.active_diff.split('\n').skip(s.diff_scroll_offset).take(diff_inner.height as usize).map(|line| {
			let style = if line.starts_with('+') && !line.starts_with("+++") {
				Style::default().fg(s.theme.success)
			} else if line.starts_with('-') && !line.starts_with("---") {
				Style::default().fg(s.theme.warning)
			} else if line.starts_with("@@") {
				Style::default().fg(s.theme.primary)
			} else if line.starts_with("commit ") || line.starts_with("diff ") || line.starts_with("Author:") || line.starts_with("Date:") {
				Style::default().fg(s.theme.accent)
			} else {
				Style::default().fg(s.theme.foreground)
			};
			Line::from(Span::styled(line, style))
		}).collect();
		
		f.render_widget(Paragraph::new(lines).style(Style::default().bg(s.theme.background)), diff_inner);

		// Bottom: Commits log
		let commits_block = Block::default()
			.borders(Borders::TOP)
			.border_style(Style::default().fg(s.theme.border))
			.title(" Recent Commit Log ")
			.title_style(if s.focus_pane == "commits" {
				Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD)
			} else {
				Style::default().fg(s.theme.foreground).add_modifier(Modifier::BOLD)
			});
		f.render_widget(commits_block.clone(), right_chunks[1]);
		let commits_inner = commits_block.inner(right_chunks[1]);

		if s.commits.is_empty() {
			f.render_widget(
				Paragraph::new("No commits found.").style(Style::default().fg(s.theme.dimmed).bg(s.theme.background)),
				commits_inner
			);
		} else {
			let items: Vec<ListItem> = s.commits.iter().enumerate().map(|(i, c)| {
				let prefix = if i == s.selected_commit_idx && s.focus_pane == "commits" { "> " } else { "  " };
				let hash_style = Style::default().fg(s.theme.accent);
				let date_style = Style::default().fg(s.theme.dimmed);
				
				let mut subject_w = right_area.width as usize - 30;
				if subject_w < 10 { subject_w = 10; }
				let mut subj = c.subject.clone();
				if subj.len() > subject_w {
					subj = format!("{}...", &subj[..subject_w - 3]);
				}

				let mut author = c.author.clone();
				if author.len() > 12 {
					author = format!("{}...", &author[..9]);
				}
				
				let item_style = if i == s.selected_commit_idx && s.focus_pane == "commits" {
					Style::default().bg(s.theme.primary).fg(s.theme.background).add_modifier(Modifier::BOLD)
				} else {
					Style::default().bg(s.theme.background)
				};

				let line = if i == s.selected_commit_idx && s.focus_pane == "commits" {
					Line::from(vec![
						Span::raw(format!("{}{}", prefix, c.hash)),
						Span::raw(format!("  ({})", c.date)),
						Span::raw(format!("  {}", subj)),
						Span::raw(format!("  [{}]", author)),
					])
				} else {
					Line::from(vec![
						Span::raw(prefix),
						Span::styled(&c.hash, hash_style),
						Span::styled(format!("  ({})", c.date), date_style),
						Span::raw(format!("  {}", subj)),
						Span::styled(format!("  [{}]", author), date_style),
					])
				};

				ListItem::new(line).style(item_style)
			}).collect();

			let list = List::new(items).style(Style::default().bg(s.theme.background));
			f.render_widget(list, commits_inner);
		}

		// ═══════════════════════════════════════════════════════════════
		// OVERLAY DE BLOQUES SÓLIDOS - Solución para Warp
		// Sobrescribimos los bordes del Block nativo con bloques sólidos (█)
		// que se renderizan como píxeles reales en cualquier emulador.
		// ═══════════════════════════════════════════════════════════════
		let overlay_style = Style::default().bg(s.theme.border).fg(s.theme.border);
		let sidebar_w_overlay: u16 = (body_rect.width / 4).max(20);

		// Calcular las posiciones de los bordes internos
		let split_x_solid = body_rect.x + sidebar_w_overlay;
		let sep_y_solid = body_rect.y + 1 + 3; // inner_y + 3 (después del info de 3 líneas)
		let right_split_y_solid = body_rect.y + 1 + ((body_rect.height - 2) * 50 / 100);

		// Borde vertical derecho del sidebar
		for row in (body_rect.y + 1)..(body_rect.y + body_rect.height) {
			f.render_widget(
				Paragraph::new("█").style(overlay_style),
				Rect { x: split_x_solid, y: row, width: 1, height: 1 }
			);
		}

		// Borde horizontal superior del files block (solo en el sidebar, no donde está el título)
		// Dejamos un hueco donde está el título "Changed Files"
		let title_gap_start = body_rect.x + 2;
		let title_gap_end = title_gap_start + 20;
		for col in body_rect.x..split_x_solid {
			if col < title_gap_start || col >= title_gap_end {
				f.render_widget(
					Paragraph::new("█").style(overlay_style),
					Rect { x: col, y: sep_y_solid, width: 1, height: 1 }
				);
			}
		}

		// Borde horizontal del diff/commits (con hueco para los títulos)
		// Título "Diff" está a la izquierda
		let right_title_gap_start = split_x_solid + 3;
		let right_title_gap_end = right_title_gap_start + 25;
		// Título "Recent Commit Log" puede estar en otra posición
		let commits_title_gap_start = split_x_solid + 3;
		let commits_title_gap_end = commits_title_gap_start + 20;
		for col in (split_x_solid + 1)..(body_rect.x + body_rect.width) {
			let in_diff_gap = col >= right_title_gap_start && col < right_title_gap_end;
			let in_commits_gap = col >= commits_title_gap_start && col < commits_title_gap_end;
			if !in_diff_gap && !in_commits_gap {
				f.render_widget(
					Paragraph::new("█").style(overlay_style),
					Rect { x: col, y: right_split_y_solid, width: 1, height: 1 }
				);
			}
		}
	} else {
		// Small screen fallback layout
		let panel_area = Rect { x: body_rect.x, y: body_rect.y + 2, width: body_rect.width, height: body_rect.height.saturating_sub(2) };
		let lines = vec![
			Line::from(format!("Dir:    {}", s.cwd)),
			Line::from(format!("Branch: {}", s.branch)),
			Line::from(format!("Remote: {}", s.remote)),
			Line::from(format!("Behind: {} commits", s.behind)),
			Line::from(format!("Ahead:  {} commits", s.ahead)),
			Line::from(format!("Files:  {} modified files", s.files.len())),
		];
		f.render_widget(Paragraph::new(lines).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)), panel_area);
	}
}

// ── Modals Drawing ───────────────────────────────────────────────────────

fn draw_setup_wizard(f: &mut Frame, s: &mut AppState) {
	let area = f.area();
	let modal_width = 60;
	let modal_height = 12;

	let modal_area = Rect {
		x: (area.width.saturating_sub(modal_width)) / 2,
		y: (area.height.saturating_sub(modal_height)) / 2,
		width: modal_width,
		height: modal_height,
	};

	let mut title = " Language Setup ";
	let mut lines = Vec::new();

	match s.setup_step {
		1 => {
			lines.push("Select Language / Selecciona Idioma:".to_string());
			lines.push("".to_string());
			let opts = vec!["English", "Español"];
			for (i, o) in opts.iter().enumerate() {
				if i == s.setup_cursor {
					lines.push(format!(" > {}", o));
				} else {
					lines.push(format!("   {}", o));
				}
			}
		}
		2 => {
			title = " Icons Setup ";
			lines.push("Select Icon Set:".to_string());
			lines.push("".to_string());
			let opts = vec!["Nerd Fonts (with icons)", "Standard ASCII (plain text)"];
			for (i, o) in opts.iter().enumerate() {
				if i == s.setup_cursor {
					lines.push(format!(" > {}", o));
				} else {
					lines.push(format!("   {}", o));
				}
			}
		}
		3 => {
			title = " Theme Setup ";
			lines.push("Select Initial Theme:".to_string());
			lines.push("".to_string());
			let themes = get_themes();
			let start = if s.setup_cursor > 3 { s.setup_cursor - 3 } else { 0 };
			let end = (start + 5).min(themes.len());
			for i in start..end {
				let t = &themes[i];
				if i == s.setup_cursor {
					lines.push(format!(" > {:<20} (preview)", t.name));
				} else {
					lines.push(format!("   {}", t.name));
				}
			}
		}
		_ => {}
	}

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(s.theme.primary))
		.title(title)
		.title_style(Style::default().fg(s.theme.primary).add_modifier(Modifier::BOLD));
	f.render_widget(Clear, modal_area);
	f.render_widget(block, modal_area);

	let content_y = modal_area.y + 2;
	f.render_widget(
		Paragraph::new(lines.join("\n")).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
		Rect { x: modal_area.x + 2, y: content_y, width: modal_area.width - 4, height: modal_area.height - 4 }
	);

	let help_str = translate(&s.language, "setup_help");
	f.render_widget(
		Paragraph::new(help_str).alignment(ratatui::layout::Alignment::Center).style(Style::default().fg(s.theme.dimmed)),
		Rect { x: modal_area.x, y: modal_area.y + modal_area.height - 2, width: modal_area.width, height: 1 }
	);
}

fn draw_theme_modal(f: &mut Frame, s: &mut AppState) {
	let area = f.area();
	let modal_width = 50;
	let themes = get_themes();
	let modal_height = themes.len() as u16 + 6;

	let modal_area = Rect {
		x: (area.width.saturating_sub(modal_width)) / 2,
		y: (area.height.saturating_sub(modal_height)) / 2,
		width: modal_width,
		height: modal_height.min(area.height - 2),
	};

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(s.theme.primary))
		.title(" Select Visual Theme ")
		.title_style(Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD));
	f.render_widget(Clear, modal_area);
	f.render_widget(block, modal_area);

	let mut lines = Vec::new();
	lines.push(format!("{}:", translate(&s.language, "theme_title")));
	lines.push("".to_string());

	let start = if s.theme_cursor > 4 { s.theme_cursor - 4 } else { 0 };
	let end = (start + 9).min(themes.len());

	for i in start..end {
		let t = &themes[i];
		if i == s.theme_cursor {
			lines.push(format!(" > {:<20} (preview)", t.name));
		} else {
			lines.push(format!("   {}", t.name));
		}
	}

	let content_y = modal_area.y + 2;
	f.render_widget(
		Paragraph::new(lines.join("\n")).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
		Rect { x: modal_area.x + 2, y: content_y, width: modal_area.width - 4, height: modal_area.height - 4 }
	);

	let help_str = translate(&s.language, "theme_help");
	f.render_widget(
		Paragraph::new(help_str).alignment(ratatui::layout::Alignment::Center).style(Style::default().fg(s.theme.dimmed)),
		Rect { x: modal_area.x, y: modal_area.y + modal_area.height - 2, width: modal_area.width, height: 1 }
	);
}

fn draw_help_modal(f: &mut Frame, s: &mut AppState) {
	let area = f.area();
	let modal_width = 55;
	let modal_height = 16;

	let modal_area = Rect {
		x: (area.width.saturating_sub(modal_width)) / 2,
		y: (area.height.saturating_sub(modal_height)) / 2,
		width: modal_width,
		height: modal_height,
	};

	let block = Block::default()
		.borders(Borders::ALL)
		.border_type(BorderType::Rounded)
		.border_style(Style::default().fg(s.theme.primary))
		.title(" Keyboard Shortcuts & Commands ")
		.title_style(Style::default().fg(s.theme.accent).add_modifier(Modifier::BOLD));
	f.render_widget(Clear, modal_area);
	f.render_widget(block, modal_area);

	let lines = if s.language == "es" {
		vec![
			"Teclas de navegación y control:",
			"",
			"  j / k / ↑ / ↓ : Mover cursor arriba/abajo",
			"  Tab           : Cambiar foco (Archivos <-> Historial)",
			"  Space / Enter : Preparar (stage) / quitar archivo",
			"  c             : Escribir commit (/commit ...)",
			"  p             : Subir cambios (git push)",
			"  f             : Descargar cambios (git fetch)",
			"  l             : Traer cambios (git pull)",
			"  t             : Cambiar tema visual (/themes)",
			"  ? / h         : Ocultar esta ayuda",
			"  q / Esc       : Salir de la aplicación",
			"  /             : Abrir barra de comandos",
			"",
			"Soporta navegación avanzada en viewport de diffs.",
		]
	} else {
		vec![
			"Navigation and control keys:",
			"",
			"  j / k / ↑ / ↓ : Navigate lists",
			"  Tab           : Switch focus (Files <-> History)",
			"  Space / Enter : Stage / Unstage file",
			"  c             : Write commit (/commit ...)",
			"  p             : Push changes (git push)",
			"  f             : Fetch changes (git fetch)",
			"  l             : Pull changes (git pull)",
			"  t             : Change visual theme (/themes)",
			"  ? / h         : Hide this help",
			"  q / Esc       : Exit application",
			"  /             : Open command bar",
			"",
			"Supports advanced diff viewport scroll.",
		]
	};

	let content_y = modal_area.y + 2;
	f.render_widget(
		Paragraph::new(lines.join("\n")).style(Style::default().fg(s.theme.foreground).bg(s.theme.background)),
		Rect { x: modal_area.x + 2, y: content_y, width: modal_area.width - 4, height: modal_area.height - 4 }
	);

	f.render_widget(
		Paragraph::new("Press any key to close.").alignment(ratatui::layout::Alignment::Center).style(Style::default().fg(s.theme.dimmed)),
		Rect { x: modal_area.x, y: modal_area.y + modal_area.height - 2, width: modal_area.width, height: 1 }
	);
}

// ── Event Handlers ───────────────────────────────────────────────────────

fn handle_key_event(code: KeyCode, s: &mut AppState) -> bool {
	// Setup Wizard Keyboard
	if s.setup_step > 0 {
		match code {
			KeyCode::Up | KeyCode::Char('k') => {
				let max = if s.setup_step == 3 { get_themes().len() - 1 } else { 1 };
				s.setup_cursor = (s.setup_cursor + max) % (max + 1);
				if s.setup_step == 3 {
					s.theme = get_themes()[s.setup_cursor];
				}
			}
			KeyCode::Down | KeyCode::Char('j') => {
				let limit = if s.setup_step == 3 { get_themes().len() } else { 2 };
				s.setup_cursor = (s.setup_cursor + 1) % limit;
				if s.setup_step == 3 {
					s.theme = get_themes()[s.setup_cursor];
				}
			}
			KeyCode::Enter => {
				match s.setup_step {
					1 => {
						s.language = if s.setup_cursor == 0 { "en".to_string() } else { "es".to_string() };
						s.setup_step = 2;
						s.setup_cursor = 0;
					}
					2 => {
						s.nerd_font = s.setup_cursor == 0;
						s.setup_step = 3;
						s.setup_cursor = 0;
					}
					3 => {
						s.theme = get_themes()[s.setup_cursor];
						s.setup_step = 0;
						s.focus_pane = "files".to_string();
						let cfg = Config {
							language: s.language.clone(),
							nerd_font: s.nerd_font,
							theme: s.theme.name.to_string(),
						};
						let _ = save_config(&cfg);
						s.refresh_git_status();
						s.status_message = translate(&s.language, "welcome_message");
					}
					_ => {}
				}
			}
			_ => {}
		}
		return true;
	}

	// Help modal keyboard
	if s.show_help_modal {
		s.show_help_modal = false;
		return true;
	}

	// Theme modal keyboard
	if s.show_theme_modal {
		let themes = get_themes();
		match code {
			KeyCode::Esc => {
				s.theme = get_theme_by_name(&s.saved_theme);
				s.show_theme_modal = false;
			}
			KeyCode::Up | KeyCode::Char('k') => {
				s.theme_cursor = (s.theme_cursor + themes.len() - 1) % themes.len();
				s.theme = themes[s.theme_cursor];
				s.update_diff_content();
			}
			KeyCode::Down | KeyCode::Char('j') => {
				s.theme_cursor = (s.theme_cursor + 1) % themes.len();
				s.theme = themes[s.theme_cursor];
				s.update_diff_content();
			}
			KeyCode::Enter => {
				s.show_theme_modal = false;
				let cfg = Config {
					language: s.language.clone(),
					nerd_font: s.nerd_font,
					theme: s.theme.name.to_string(),
				};
				let _ = save_config(&cfg);
				s.status_message = format!("Theme changed to: {}", s.theme.name);
			}
			_ => {}
		}
		return true;
	}

	// Git Init Wizard Keyboard
	if s.init_wizard_active {
		if (s.init_wizard_step == 1 && s.init_cursor == 2) || s.init_wizard_step == 2 {
			match code {
				KeyCode::Esc => {
					s.init_wizard_active = false;
					s.status_message = "Git initialization cancelled.".to_string();
				}
				KeyCode::Enter => {
					if s.init_wizard_step == 1 {
						if s.input_value.is_empty() {
							s.status_message = "Branch name cannot be empty.".to_string();
							return true;
						}
						s.init_branch_name = s.input_value.clone();
						s.init_wizard_step = 2;
						s.input_value.clear();
						s.input_cursor_pos = 0;
					} else {
						s.init_remote_url = s.input_value.clone();
						s.init_wizard_step = 3;
						s.init_cursor = 0;
						s.input_value.clear();
						s.input_cursor_pos = 0;
					}
				}
				KeyCode::Backspace => {
					if s.input_cursor_pos > 0 {
						s.input_value.remove(s.input_cursor_pos - 1);
						s.input_cursor_pos -= 1;
					}
				}
				KeyCode::Left => {
					if s.input_cursor_pos > 0 {
						s.input_cursor_pos -= 1;
					}
				}
				KeyCode::Right => {
					if s.input_cursor_pos < s.input_value.len() {
						s.input_cursor_pos += 1;
					}
				}
				KeyCode::Char(c) => {
					s.input_value.insert(s.input_cursor_pos, c);
					s.input_cursor_pos += 1;
				}
				_ => {}
			}
			return true;
		}

		match code {
			KeyCode::Esc => {
				s.init_wizard_active = false;
				s.status_message = "Git initialization cancelled.".to_string();
			}
			KeyCode::Up | KeyCode::Char('k') => {
				if s.init_wizard_step == 1 {
					s.init_cursor = (s.init_cursor + 2) % 3;
				} else if s.init_wizard_step == 3 {
					s.init_cursor = (s.init_cursor + 1) % 2;
				}
			}
			KeyCode::Down | KeyCode::Char('j') => {
				if s.init_wizard_step == 1 {
					s.init_cursor = (s.init_cursor + 1) % 3;
				} else if s.init_wizard_step == 3 {
					s.init_cursor = (s.init_cursor + 1) % 2;
				}
			}
			KeyCode::Enter => {
				if s.init_wizard_step == 1 {
					if s.init_cursor == 0 {
						s.init_branch_name = "main".to_string();
						s.init_wizard_step = 2;
						s.input_value.clear();
						s.input_cursor_pos = 0;
					} else if s.init_cursor == 1 {
						s.init_branch_name = "master".to_string();
						s.init_wizard_step = 2;
						s.input_value.clear();
						s.input_cursor_pos = 0;
					} else {
						s.input_value.clear();
						s.input_cursor_pos = 0;
					}
				} else if s.init_wizard_step == 3 {
					if s.init_cursor == 0 {
						s.init_wizard_active = false;
						let _ = git::run_git(&["init"]);
						let _ = git::run_git(&["checkout", "-b", &s.init_branch_name]);
						if !s.init_remote_url.is_empty() {
							let _ = git::run_git(&["remote", "add", "origin", &s.init_remote_url]);
						}
						let _ = git::git_add_all();
						let _ = git::git_commit("Initial commit");
						s.refresh_git_status();
						s.status_message = "Git repository initialized successfully!".to_string();
					} else {
						s.init_wizard_active = false;
						s.status_message = "Git initialization cancelled.".to_string();
					}
				}
			}
			_ => {}
		}
		return true;
	}

	// Commands Input Mode keyboard
	if s.show_input {
		match code {
			KeyCode::Esc => {
				s.show_input = false;
				s.input_value.clear();
				s.input_cursor_pos = 0;
				s.suggestions.clear();
			}
			KeyCode::Tab => {
				if !s.suggestions.is_empty() {
					s.input_value = s.suggestions[s.active_sug].clone();
					s.input_cursor_pos = s.input_value.len();
					s.update_suggestions();
				}
			}
			KeyCode::Up => {
				if !s.suggestions.is_empty() {
					s.active_sug = (s.active_sug + s.suggestions.len() - 1) % s.suggestions.len();
				}
			}
			KeyCode::Down => {
				if !s.suggestions.is_empty() {
					s.active_sug = (s.active_sug + 1) % s.suggestions.len();
				}
			}
			KeyCode::Enter => {
				let cmd = s.input_value.clone();
				s.show_input = false;
				s.input_value.clear();
				s.input_cursor_pos = 0;
				s.suggestions.clear();
				s.execute_command(&cmd);
			}
			KeyCode::Backspace => {
				if s.input_cursor_pos > 0 {
					s.input_value.remove(s.input_cursor_pos - 1);
					s.input_cursor_pos -= 1;
					s.update_suggestions();
				}
			}
			KeyCode::Left => {
				if s.input_cursor_pos > 0 {
					s.input_cursor_pos -= 1;
				}
			}
			KeyCode::Right => {
				if s.input_cursor_pos < s.input_value.len() {
					s.input_cursor_pos += 1;
				}
			}
			KeyCode::Char(c) => {
				s.input_value.insert(s.input_cursor_pos, c);
				s.input_cursor_pos += 1;
				s.update_suggestions();
			}
			_ => {}
		}
		return true;
	}

	// Normal Mode keyboard (Not in a Git repo)
	if !s.is_git_repo {
		match code {
			KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k') => {
				s.init_cursor = 1 - s.init_cursor;
			}
			KeyCode::Enter => {
				if s.init_cursor == 0 {
					s.init_wizard_active = true;
					s.init_wizard_step = 1;
					s.init_cursor = 0;
					s.init_branch_name = "main".to_string();
					s.init_remote_url.clear();
					s.status_message = "Select main branch name.".to_string();
				} else {
					s.show_input = true;
					s.input_value = "/cd ".to_string();
					s.input_cursor_pos = 4;
					s.update_suggestions();
				}
			}
			KeyCode::Char('t') | KeyCode::Char('T') => {
				s.execute_command("/themes");
			}
			KeyCode::Char('q') | KeyCode::Char('Q') => {
				return false;
			}
			_ => {}
		}
		return true;
	}

	// Normal Mode keyboard (Inside Git repo)
	match code {
		KeyCode::Tab => {
			if s.focus_pane == "files" {
				s.focus_pane = "commits".to_string();
			} else {
				s.focus_pane = "files".to_string();
			}
			s.diff_scroll_offset = 0;
			s.update_diff_content();
		}
		KeyCode::Up | KeyCode::Char('k') => {
			if s.focus_pane == "files" && !s.files.is_empty() {
				s.selected_file_idx = (s.selected_file_idx + s.files.len() - 1) % s.files.len();
				s.update_diff_content();
				s.diff_scroll_offset = 0;
			} else if s.focus_pane == "commits" && !s.commits.is_empty() {
				s.selected_commit_idx = (s.selected_commit_idx + s.commits.len() - 1) % s.commits.len();
				s.update_diff_content();
				s.diff_scroll_offset = 0;
			}
		}
		KeyCode::Down | KeyCode::Char('j') => {
			if s.focus_pane == "files" && !s.files.is_empty() {
				s.selected_file_idx = (s.selected_file_idx + 1) % s.files.len();
				s.update_diff_content();
				s.diff_scroll_offset = 0;
			} else if s.focus_pane == "commits" && !s.commits.is_empty() {
				s.selected_commit_idx = (s.selected_commit_idx + 1) % s.commits.len();
				s.update_diff_content();
				s.diff_scroll_offset = 0;
			}
		}
		KeyCode::Char(' ') | KeyCode::Enter => {
			if s.focus_pane == "files" && !s.files.is_empty() {
				s.toggle_stage_file(s.selected_file_idx);
			}
		}
		KeyCode::Char('c') | KeyCode::Char('C') => {
			s.show_input = true;
			s.input_value = "/commit ".to_string();
			s.input_cursor_pos = 8;
		}
		KeyCode::Char('p') | KeyCode::Char('P') => {
			s.execute_command("/push");
		}
		KeyCode::Char('f') | KeyCode::Char('F') => {
			s.execute_command("/fetch");
		}
		KeyCode::Char('l') | KeyCode::Char('L') => {
			s.execute_command("/pull");
		}
		KeyCode::Char('t') | KeyCode::Char('T') => {
			s.execute_command("/themes");
		}
		KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
			s.show_help_modal = true;
		}
		KeyCode::Char('q') | KeyCode::Char('Q') => {
			return false;
		}
		KeyCode::Char('/') => {
			s.show_input = true;
			s.input_value = "/".to_string();
			s.input_cursor_pos = 1;
			s.update_suggestions();
		}
		KeyCode::PageDown => {
			s.diff_scroll_offset += 5;
		}
		KeyCode::PageUp => {
			if s.diff_scroll_offset >= 5 {
				s.diff_scroll_offset -= 5;
			} else {
				s.diff_scroll_offset = 0;
			}
		}
		_ => {}
	}

	true
}

fn handle_mouse_click(col: u16, row: u16, s: &mut AppState, terminal: &Terminal<CrosstermBackend<Stdout>>) {
	let area = terminal.size().unwrap_or_default();
	
	let margin_x = if area.width < 25 { 1 } else { 2 };
	let margin_y = if area.height < 10 { 0 } else { 1 };

	let outer_rect = Rect {
		x: margin_x,
		y: margin_y,
		width: area.width.saturating_sub(margin_x * 2),
		height: area.height.saturating_sub(margin_y * 2),
	};

	// inner_rect is the area inside the outer block border
	let inner_rect = Rect {
		x: outer_rect.x + 1,
		y: outer_rect.y + 1,
		width: outer_rect.width.saturating_sub(2),
		height: outer_rect.height.saturating_sub(2),
	};

	// Close input click outside
	if s.show_input {
		let input_y = inner_rect.y + inner_rect.height - 1;
		let sug_len = s.suggestions.len() as u16;
		let sug_start_y = input_y.saturating_sub(sug_len) - 1;

		let clicked_input = row == input_y && col >= inner_rect.x && col < inner_rect.x + inner_rect.width;
		let clicked_sug = sug_len > 0 && row >= sug_start_y && row < input_y && col >= inner_rect.x;

		if !clicked_input && !clicked_sug {
			s.show_input = false;
			s.input_value.clear();
			s.input_cursor_pos = 0;
			s.suggestions.clear();
			return;
		}
	}

	// 1. Setup Wizard Clicks
	if s.setup_step > 0 {
		let modal_width = 60;
		let modal_height = 12;
		let modal_x = (area.width.saturating_sub(modal_width)) / 2;
		let modal_y = (area.height.saturating_sub(modal_height)) / 2;

		if col >= modal_x + 2 && col < modal_x + modal_width - 2 && row >= modal_y + 2 && row < modal_y + modal_height - 2 {
			let idx = row.saturating_sub(modal_y + 2) as usize;
			match s.setup_step {
				1 => {
					if idx < 2 { s.setup_cursor = idx; }
				}
				2 => {
					if idx < 2 { s.setup_cursor = idx; }
				}
				3 => {
					let themes = get_themes();
					let start = if s.setup_cursor > 3 { s.setup_cursor - 3 } else { 0 };
					let clicked = start + idx;
					if clicked < themes.len() { s.setup_cursor = clicked; }
				}
				_ => {}
			}
		}
		return;
	}

	// 2. Help Modal Close on click
	if s.show_help_modal {
		s.show_help_modal = false;
		return;
	}

	// 3. Theme Modal Clicks
	if s.show_theme_modal {
		let modal_width = 50;
		let themes = get_themes();
		let modal_height = themes.len() as u16 + 6;
		let modal_x = (area.width.saturating_sub(modal_width)) / 2;
		let modal_y = (area.height.saturating_sub(modal_height)) / 2;

		if col >= modal_x + 2 && col < modal_x + modal_width - 2 && row >= modal_y + 2 && row < modal_y + modal_height - 3 {
			let start = if s.theme_cursor > 4 { s.theme_cursor - 4 } else { 0 };
			let idx = start + row.saturating_sub(modal_y + 2) as usize;
			if idx < themes.len() {
				s.theme_cursor = idx;
				s.theme = themes[idx];
				s.update_diff_content();
			}
		}
		return;
	}

	// 4. Git Init Wizard Clicks
	if s.init_wizard_active {
		let panel_y = inner_rect.y + 1;
		let clicked = row.saturating_sub(panel_y + 2);

		match s.init_wizard_step {
			1 => {
				if clicked >= 3 && clicked <= 5 {
					let idx = clicked as usize - 3;
					s.init_cursor = idx;
					if idx == 0 {
						s.init_branch_name = "main".to_string();
						s.init_wizard_step = 2;
						s.input_value.clear();
					} else if idx == 1 {
						s.init_branch_name = "master".to_string();
						s.init_wizard_step = 2;
						s.input_value.clear();
					}
				}
			}
			3 => {
				if clicked == 7 {
					s.init_cursor = 0;
					// Initialize repo
					s.init_wizard_active = false;
					let _ = git::run_git(&["init"]);
					let _ = git::run_git(&["checkout", "-b", &s.init_branch_name]);
					if !s.init_remote_url.is_empty() {
						let _ = git::run_git(&["remote", "add", "origin", &s.init_remote_url]);
					}
					let _ = git::git_add_all();
					let _ = git::git_commit("Initial commit");
					s.refresh_git_status();
					s.status_message = "Git repository initialized successfully!".to_string();
				} else if clicked == 8 {
					s.init_cursor = 1;
					s.init_wizard_active = false;
					s.status_message = "Git initialization cancelled.".to_string();
				}
			}
			_ => {}
		}
		return;
	}

	// 5. No repository panel click options
	if !s.is_git_repo {
		let panel_y = inner_rect.y + 1;
		let content_y = panel_y + 3;
		if row == content_y + 6 {
			s.init_cursor = 0;
			s.init_wizard_active = true;
			s.init_wizard_step = 1;
			s.init_branch_name = "main".to_string();
			s.init_remote_url.clear();
			s.status_message = "Select main branch name.".to_string();
		} else if row == content_y + 8 {
			s.init_cursor = 1;
			s.show_input = true;
			s.input_value = "/cd ".to_string();
			s.input_cursor_pos = 4;
			s.update_suggestions();
		}
		return;
	}

	// 6. Interactive Dashboard clicks
	let sidebar_w = 32;
	let split_x = inner_rect.x + sidebar_w;
	let sep_y = inner_rect.y + 6;
	let file_list_start_y = sep_y + 1;

	if col >= inner_rect.x && col < split_x {
		// Files sidebar list clicked
		let clicked_idx = row.saturating_sub(file_list_start_y) as usize;
		if clicked_idx < s.files.len() {
			s.focus_pane = "files".to_string();
			s.selected_file_idx = clicked_idx;
			s.diff_scroll_offset = 0;

			// Click checkbox column (check/uncheck)
			if col >= inner_rect.x + 2 && col <= inner_rect.x + 6 {
				s.toggle_stage_file(clicked_idx);
			} else {
				s.update_diff_content();
			}
		}
	} else if col > split_x && col < inner_rect.x + inner_rect.width {
		// Commits history log list clicked
		let split_y = self_split_y(inner_rect);
		let commit_list_start_y = split_y + 2;
		let clicked_idx = row.saturating_sub(commit_list_start_y) as usize;
		if clicked_idx < s.commits.len() {
			s.focus_pane = "commits".to_string();
			s.selected_commit_idx = clicked_idx;
			s.diff_scroll_offset = 0;
			s.update_diff_content();
		}
	}
}

fn self_split_y(inner_rect: Rect) -> u16 {
	let right_h = inner_rect.height.saturating_sub(3);
	inner_rect.y + 1 + (right_h * 50 / 100)
}

// ── Path Expanding Helpers ────────────────────────────────────────────────

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
					if prefix.is_empty() || name.to_lowercase().starts_with(&prefix.to_lowercase()) {
						let mut base_path = path_arg.to_string();
						if !prefix.is_empty() {
							base_path = path_arg[..path_arg.len() - prefix.len()].to_string();
						}
						if !base_path.is_empty() && !base_path.ends_with('/') && !base_path.ends_with('\\') {
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
		"/cd ".to_string(),
		"/themes".to_string(),
		"/help".to_string(),
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
