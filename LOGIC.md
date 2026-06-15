# Equivalencias de Lógica: de Zsh a Rust 🦀

Este documento desglosa la lógica exacta de tu script `gacp` original y proporciona su equivalente en Rust para que puedas implementarlo paso a paso.

---

## 🛠️ Utilidad Base: Ejecución de Comandos Git en Rust

Para evitar repetir código, en Rust necesitarás una función auxiliar que ejecute un comando `git` en la terminal, capture su salida (stdout) o errores (stderr), y los devuelva de forma procesada.

```rust
use std::process::Command;
use std::io::{self, Error, ErrorKind};

/// Ejecuta un comando git con los argumentos provistos y devuelve la salida como String.
fn run_git(args: &[&str]) -> io::Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()?; // Ejecuta y bloquea hasta terminar

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(Error::new(ErrorKind::Other, err_msg))
    }
}
```

---

## 1. Verificación del Repositorio Git

### Script Original (Zsh):
```zsh
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
  echo -e "\033[0;31m  ✖  No estas dentro de un repositorio git.\033[0m"
  return 1
fi
```

### Equivalente en Rust:
```rust
fn is_inside_work_tree() -> bool {
    run_git(&["rev-parse", "--is-inside-work-tree"]).is_ok()
}
```

---

## 2. Obtener Rama y Remoto

### Script Original (Zsh):
```zsh
local branch=$(git symbolic-ref --short HEAD 2>/dev/null)
local remote=$(git config branch.$branch.remote 2>/dev/null || echo "origin")
```

### Equivalente en Rust:
```rust
fn get_current_branch() -> io::Result<String> {
    run_git(&["symbolic-ref", "--short", "HEAD"])
}

fn get_remote(branch: &str) -> String {
    let config_key = format!("branch.{}.remote", branch);
    run_git(&["config", &config_key])
        .unwrap_or_else(|_| "origin".to_string())
}
```

---

## 3. Descarga (Fetch) y Conteo de Commits (*Ahead / Behind*)

### Script Original (Zsh):
```zsh
git fetch $remote $branch 2>/dev/null
local behind=$(git rev-list --count HEAD..$remote/$branch 2>/dev/null)
local ahead=$(git rev-list --count $remote/$branch..HEAD 2>/dev/null)
```

### Equivalente en Rust:
```rust
// 1. Ejecutar el fetch (puede tardar un poco, no captura salida)
fn fetch_remote(remote: &str, branch: &str) -> io::Result<()> {
    run_git(&["fetch", remote, branch])?;
    Ok(())
}

// 2. Obtener commits pendientes por descargar (behind)
fn get_commits_behind(remote: &str, branch: &str) -> usize {
    let range = format!("HEAD..{}/{}", remote, branch);
    run_git(&["rev-list", "--count", &range])
        .ok()
        .and_then(|out| out.parse::<usize>().ok())
        .unwrap_or(0)
}

// 3. Obtener commits pendientes por subir (ahead)
fn get_commits_ahead(remote: &str, branch: &str) -> usize {
    let range = format!("{}/{}..HEAD", remote, branch);
    run_git(&["rev-list", "--count", &range])
        .ok()
        .and_then(|out| out.parse::<usize>().ok())
        .unwrap_or(0)
}
```

---

## 4. Verificar Cambios sin Commitear (Status)

### Script Original (Zsh):
```zsh
if [ -n "$(git status --porcelain)" ]; then
  # Hay cambios pendientes
fi
```

### Equivalente en Rust:
```rust
fn has_uncommitted_changes() -> bool {
    match run_git(&["status", "--porcelain"]) {
        Ok(out) => !out.is_empty(),
        Err(_) => false,
    }
}

// Para la TUI, querrás listar los archivos modificados:
fn get_modified_files() -> io::Result<Vec<String>> {
    let output = run_git(&["status", "--porcelain"])?;
    if output.is_empty() {
        return Ok(vec![]);
    }
    let files = output
        .lines()
        .map(|line| line.to_string())
        .collect();
    Ok(files)
}
```

---

## 5. Add, Commit, Pull y Push Interactivo

En Rust, para la entrada de datos en consola (como leer el mensaje del commit o confirmar acciones con `s/N`), puedes usar `std::io::stdin()` en el modo CLI clásico, o capturar las teclas usando `crossterm` si ya estás en el modo TUI.

### Ejemplo de flujo interactivo (Consola Estándar):

```rust
use std::io::{self, Write};

fn prompt_commit_message() -> io::Result<String> {
    print!("  💬 Mensaje del commit:\n  → ");
    io::stdout().flush()?; // Forzar dibujado en consola
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let msg = input.trim().to_string();
    
    if msg.is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "Mensaje vacío"));
    }
    Ok(msg)
}

fn confirm_action(prompt: &str) -> bool {
    print!("{} (s/N): ", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let answer = input.trim().to_lowercase();
        answer == "s" || answer == "si"
    } else {
        false
    }
}
```

### Ejecutar las operaciones Git destructivas/modificadoras:
```rust
fn git_add_all() -> io::Result<String> {
    run_git(&["add", "."])
}

fn git_commit(message: &str) -> io::Result<String> {
    run_git(&["commit", "-m", message])
}

fn git_pull(remote: &str, branch: &str) -> io::Result<String> {
    run_git(&["pull", remote, branch])
}

fn git_push(remote: &str, branch: &str) -> io::Result<String> {
    run_git(&["push", remote, branch])
}
```
