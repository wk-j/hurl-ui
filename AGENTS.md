# AGENTS.md - AI Coding Agent Guidelines

This document provides guidelines for AI coding agents working on the hurl-tui codebase.

## Project Overview

Hurl TUI is a Rust terminal user interface for [Hurl](https://hurl.dev/) HTTP testing. It uses `ratatui` for the TUI framework and `tokio` for async runtime.

## Build, Test, and Lint Commands

```bash
cargo build                      # Build development
cargo build --release            # Build release (optimized)
cargo run -- /path/to/files      # Run with directory

cargo test                       # Run all tests
cargo test test_parse_simple     # Run single test by name
cargo test parser::tests         # Run tests in module
cargo test -- --nocapture        # Run with output visible

cargo fmt                        # Format code
cargo clippy                     # Lint
cargo check                      # Check without building
```

## Project Structure

```
hurl-tui/
├── src/
│   ├── main.rs           # Entry point, terminal setup
│   ├── app.rs            # Core application state (largest file)
│   ├── config/mod.rs     # Configuration structs and loading
│   ├── events/mod.rs     # Event handling (Key, Mouse, Tick)
│   ├── parser/mod.rs     # Hurl file parsing
│   ├── runner/mod.rs     # Hurl execution via CLI
│   └── ui/
│       ├── mod.rs        # UI exports and main draw()
│       ├── layout.rs     # Layout management
│       ├── file_browser.rs, editor.rs, response.rs
│       ├── assertions.rs, variables.rs
│       ├── status_bar.rs, help.rs
├── examples/             # Sample .hurl files for testing
└── Cargo.toml
```

## Code Style Guidelines

### Import Ordering

Order imports with blank lines between groups:
1. Standard library (`use std::...`)
2. External crates (`use anyhow::...`, `use serde::...`)
3. Internal crate modules (`use crate::...`)

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Structs/Enums | PascalCase | `ExecutionResult`, `ActivePanel` |
| Functions/Variables | snake_case | `parse_hurl_file`, `file_tree` |
| Modules | snake_case | `file_browser`, `status_bar` |

### Error Handling

Use `anyhow::Result<T>` with context:
```rust
std::fs::read_to_string(path).context("Failed to read file")?
```

Use `let-else` for early returns:
```rust
let Some(path) = self.current_file_path.clone() else {
    self.set_status("No file selected", StatusLevel::Warning);
    return Ok(());
};
```

Silent failure for non-critical operations:
```rust
if let Ok(content) = serde_json::to_string_pretty(&state) {
    let _ = std::fs::write(path, content);
}
```

### Type Patterns

Derive common traits for public types:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult { ... }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePanel {
    #[default]
    FileBrowser,
    Editor,
}
```

### Testing

Place tests inline using `#[cfg(test)]`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_request() {
        let result = parse_hurl_file(content).unwrap();
        assert_eq!(result.entries.len(), 1);
    }
}
```

## Important Guidelines

### Documentation Updates

**MANDATORY: Always update documentation after implementing new features:**
- Update `README.md` (root) for user-facing feature changes
- Update `hurl-tui/README.md` for detailed documentation
- Update help text in `ui/help.rs` for keyboard shortcuts
- This is a required step, not optional - features are not complete until documented

### State Persistence

The app persists state to `.hurl-tui-state.json`:
- Last opened file and file tree index
- Execution results per file (keyed by relative path)

When modifying state, update `PersistedState` struct, `save_state()`, and `restore_last_opened_file()`.

### Key Handling

Key events handled in `app.rs` by mode:
- `handle_normal_mode_key()` - Navigation
- `handle_editing_mode_key()` - Text editing
- `handle_search_mode_key()` - Search input
- `handle_command_mode_key()` - Vim-style commands

## Common Patterns

### Adding a new field to App state

1. Add field to `App` struct in `app.rs`
2. Initialize in `App::new()`
3. If persisted, add to `PersistedState` struct
4. Update `save_state()` and `restore_last_opened_file()`

### Adding a new keyboard shortcut

1. Add match arm in appropriate handler (`handle_normal_mode_key`, etc.)
2. Implement the action method
3. Update help text in `ui/help.rs`
4. Document in README.md keyboard shortcuts table

### Adding a new UI panel

1. Create `src/ui/new_panel.rs`
2. Add `pub mod new_panel;` to `src/ui/mod.rs`
3. Export render function: `pub use new_panel::render_new_panel;`
4. Add variant to `ActivePanel` enum
5. Update `next_panel()` / `previous_panel()` navigation
6. Call render function from `ui::draw()`

