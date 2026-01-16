# Hurl TUI

A Terminal User Interface for [Hurl](https://hurl.dev/), the command-line tool for running HTTP requests defined in simple plain text format.

## Features

- **File Browser**: Navigate and manage `.hurl` files in a tree view
- **Syntax Highlighting**: Hurl-specific syntax highlighting in the editor
- **Request Execution**: Run Hurl requests directly from the TUI
- **Response Viewer**: View formatted responses with JSON pretty-printing
- **Assertions Panel**: See assertion results with pass/fail status
- **Environment Variables**: Manage and switch between environments
- **Vim-style Navigation**: Familiar keyboard shortcuts for efficient navigation

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/hurl-tui.git
cd hurl-tui

# Build and install
cargo install --path .
```

### Requirements

- Rust 1.75 or later
- [Hurl](https://hurl.dev/) installed and available in PATH

## Usage

```bash
# Run in current directory
hurl-tui

# Run in a specific directory
hurl-tui /path/to/hurl/files
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `h` | Go to file browser |
| `l` | Go to editor/response |
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

### Actions

| Key | Action |
|-----|--------|
| `Enter` | Open file / Run request |
| `Space` | Expand/collapse folder |
| `r` | Run current request |
| `e` | Enter edit mode |
| `v` | Toggle variables panel |
| `E` | Cycle environment |
| `R` | Refresh file tree |
| `/` | Search files |
| `:` | Command mode |
| `?` | Show help |
| `q` | Quit |

### Commands

| Command | Action |
|---------|--------|
| `:w` | Save file |
| `:q` | Quit |
| `:wq` | Save and quit |

## Configuration

Create a configuration file at `~/.config/hurl-tui/config.toml`:

```toml
[general]
timeout = 30
max_history = 100

[ui]
show_line_numbers = true
show_icons = true
theme = "default"

[editor]
tab_size = 2
use_spaces = true
auto_save = false
```

## Project Structure

```
hurl-tui/
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Application state
│   ├── config/          # Configuration management
│   ├── events/          # Input event handling
│   ├── parser/          # Hurl file parsing
│   ├── runner/          # Hurl execution
│   └── ui/              # UI components
├── config/
│   └── default.toml     # Default configuration
└── Cargo.toml
```

## Development

```bash
# Run in development mode
cargo run

# Run with a specific directory
cargo run -- /path/to/hurl/files

# Run tests
cargo test

# Build release
cargo build --release
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
