# Hurl TUI

A terminal user interface for [Hurl](https://hurl.dev/) - run and debug HTTP requests from your terminal.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- Browse and manage `.hurl` files
- Syntax highlighting for Hurl format
- Execute requests and view responses
- JSON pretty-printing
- Assertion results with pass/fail status
- Environment variable management
- Vim-style keyboard navigation

## Quick Start

```bash
# Build and run
cd hurl-tui
cargo run

# Or run in a specific directory
cargo run -- /path/to/hurl/files
```

**Requirements:** Rust 1.75+, [Hurl](https://hurl.dev/) CLI

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j/k` | Navigate up/down |
| `Enter` | Open file / Run request |
| `r` | Run request |
| `e` | Edit mode |
| `Tab` | Cycle panels |
| `?` | Help |
| `q` | Quit |

## Project Structure

```
hurl-tui/
├── PROPOSAL.md    # Technical design document
├── README.md      # Detailed documentation
└── src/           # Source code
```

## License

MIT
