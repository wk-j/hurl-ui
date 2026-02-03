# Hurl TUI

A Terminal User Interface for [Hurl](https://hurl.dev/), the command-line tool for running HTTP requests defined in simple plain text format.

## Features

- **File Browser**: Navigate and manage `.hurl` files in a tree view with auto-expand
- **File Copy/Paste**: Duplicate files with `p` (copy) and `P` (paste)
- **File Rename**: Rename files with `n` key
- **File Filtering**: Filter files by name with `f` key, clear with `F`
- **Auto Preview**: Automatically preview files when navigating
- **Syntax Highlighting**: Hurl-specific syntax highlighting in the editor
- **Request Execution**: Run Hurl requests directly from the TUI
- **Response Viewer**: View formatted responses with JSON pretty-printing
- **Assertions Panel**: See assertion results with pass/fail status
- **Environment Variables**: Manage and switch between environments with `.env` files
- **Vim-style Navigation**: Familiar keyboard shortcuts for efficient navigation
- **Clipboard Support**: Copy file paths and responses to clipboard
- **Session Restore**: Remember last opened file per directory
- **Execution State Persistence**: Cached execution results per file are restored across sessions

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
| `W` | Run & write output to file |
| `e` | Enter edit mode (vim) |
| `v` | Toggle variables panel |
| `E` | Cycle environment |
| `R` | Refresh file tree |
| `/` | Search files |
| `f` | Filter files by name |
| `F` | Clear filter |
| `p` | Copy file (for paste) |
| `P` | Paste copied file |
| `n` | Rename file |
| `[` / `]` | Resize sidebar |
| `A` | Toggle assertions panel |
| `:` | Command mode |
| `?` | Show help |
| `q` | Quit |

### Vim Edit Mode

When in edit mode (`e`), the editor uses vim-style keybindings:

**Normal Mode** (navigation & commands):

| Key | Action |
|-----|--------|
| `h/j/k/l` | Move cursor left/down/up/right |
| `w/b/e` | Word forward/backward/end |
| `0/$` | Line start/end |
| `^` | First non-whitespace |
| `g/G` | First/last line |
| `Ctrl+d/u` | Page down/up |
| `i/a` | Insert before/after cursor |
| `I/A` | Insert at line start/end |
| `o/O` | Open line below/above |
| `x` | Delete character |
| `d` | Delete line |
| `D` | Delete to end of line |
| `Esc/q` | Exit edit mode |

**Insert Mode** (text input):

| Key | Action |
|-----|--------|
| (typing) | Insert characters |
| `Esc` | Return to Normal mode |
| `Backspace` | Delete before cursor |
| Arrow keys | Move cursor |

### Clipboard & Output

| Key | Action |
|-----|--------|
| `y` | Copy file path to clipboard |
| `Y` | Copy response to clipboard |
| `W` | Run & write output to file (e.g., `test.hurl` -> `test.output`) |
| `c` | Copy AI context (request + response + assertions) |
| `C` | Copy hurl command to clipboard |
| `o` | Output AI context to stdout and quit |

### AI Context Format

Press `c` to copy the full test context in markdown format, ideal for AI prompts:

```markdown
## Hurl Test: api/users.hurl

### Request (Hurl file)
```hurl
GET https://api.example.com/users
HTTP 200
[Asserts]
jsonpath "$" count > 0
```

### Response
**Status:** 200
**Duration:** 145ms

**Body:**
```json
{ "users": [...] }
```

### Assertion Results
| Status | Assertion |
|--------|----------|
| PASS | jsonpath "$" count > 0 |

### Result: SUCCESS
```

### Commands

| Command | Action |
|---------|--------|
| `:w` | Save file |
| `:q` | Quit |
| `:wq` | Save and quit |

## Helix Editor Integration

Add to your `~/.config/helix/config.toml`:

```toml
[keys.normal]
H = [
    ":new",
    ":insert-output hurl-tui",
    ":buffer-close!",
    ":redraw"
]
```

Press `H` in Helix to open hurl-tui. Navigate to a `.hurl` file, run it with `r`, then press `o` to output the AI context to your buffer.

The TUI works interactively even when stdout is piped, using `/dev/tty` for terminal access.

## Environment Variables

Place `.env` files anywhere in your project directory to define environment-specific variables:

```
my-project/
├── api/
│   └── users.hurl
├── env/
│   ├── local.env
│   ├── staging.env
│   └── production.env
└── tests/
    └── integration.hurl
```

Example `.env` file:
```env
# local.env
base_url=http://localhost:8080
api_key=dev-key-123
timeout=30
```

Use variables in your `.hurl` files:
```hurl
GET {{base_url}}/users
Authorization: Bearer {{api_key}}
HTTP 200
```

- Press `E` to cycle between available environments
- The selected environment is persisted across sessions
- Variables are passed to hurl using `--variables-file`

## Configuration

Create a configuration file at `~/.config/hurl-tui/config.toml`:

```toml
[general]
timeout = 30
max_history = 100
# Optional: directory for output files (default: same as hurl file)
# output_dir = "/path/to/outputs"

[ui]
show_line_numbers = true
show_icons = true
theme = "default"

[editor]
tab_size = 2
use_spaces = true
auto_save = false
```

### Output Files

Press `W` to run the current request and write the response body to a file. The output file is automatically named after the hurl file:

- `api/users.hurl` -> `api/users.output`

If `output_dir` is configured, all output files are written there instead:

```toml
[general]
output_dir = "outputs"
```

This would write to `outputs/users.output` regardless of where the hurl file is located.

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
├── examples/            # Sample hurl files
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
