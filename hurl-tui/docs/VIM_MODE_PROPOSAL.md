# Proposal: Minimal Vim Mode for Editor

## Overview

Implement basic Vim-style modal editing with Normal and Insert modes, providing familiar keybindings for Vim users while keeping the implementation simple.

## Scope

### In Scope (Minimal Vim)

- **Modal editing**: Normal mode and Insert mode
- **Navigation** (Normal mode): `h`, `j`, `k`, `l`, `0`, `$`, `gg`, `G`, `w`, `b`
- **Insert commands**: `i`, `a`, `o`, `O`, `I`, `A`
- **Delete commands**: `x`, `dd`, `D`
- **Mode switching**: `Esc` (to Normal), `i/a/o/O/I/A` (to Insert)
- **Visual feedback**: Show current vim mode in status bar and editor

### Out of Scope (Future Enhancements)

- Visual mode (`v`, `V`)
- Undo/redo (`u`, `Ctrl+r`)
- Yank/paste (`y`, `p`, `P`)
- Registers
- Macros
- Count prefixes (`5j`, `3dd`)
- Text objects (`iw`, `aw`, `i"`, etc.)
- Search within editor (`/`, `n`, `N`)
- Replace mode (`R`)

---

## Technical Design

### 1. New Types

```rust
// src/app.rs

/// Vim sub-mode when in Editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimMode {
    #[default]
    Normal,   // Navigation and commands
    Insert,   // Text input
}
```

### 2. App State Changes

```rust
// Add to App struct
pub struct App {
    // ... existing fields ...
    
    /// Current vim mode (when in Editing mode)
    pub vim_mode: VimMode,
}
```

### 3. Key Handler Refactoring

Replace `handle_editing_mode_key()` with vim-aware handling:

```rust
fn handle_editing_mode_key(&mut self, key: KeyEvent) -> Result<()> {
    match self.vim_mode {
        VimMode::Normal => self.handle_vim_normal_mode(key),
        VimMode::Insert => self.handle_vim_insert_mode(key),
    }
}

fn handle_vim_normal_mode(&mut self, key: KeyEvent) -> Result<()> {
    match key.code {
        // Exit editing
        KeyCode::Esc | KeyCode::Char('q') => {
            self.mode = AppMode::Normal;
            self.vim_mode = VimMode::Normal;
        }
        
        // Navigation
        KeyCode::Char('h') | KeyCode::Left => self.editor_move_cursor_left(),
        KeyCode::Char('j') | KeyCode::Down => self.editor_move_cursor_down(),
        KeyCode::Char('k') | KeyCode::Up => self.editor_move_cursor_up(),
        KeyCode::Char('l') | KeyCode::Right => self.editor_move_cursor_right(),
        KeyCode::Char('0') | KeyCode::Home => self.editor_move_to_line_start(),
        KeyCode::Char('$') | KeyCode::End => self.editor_move_to_line_end(),
        KeyCode::Char('g') => self.editor_move_to_first_line(), // simplified gg
        KeyCode::Char('G') => self.editor_move_to_last_line(),
        KeyCode::Char('w') => self.editor_move_word_forward(),
        KeyCode::Char('b') => self.editor_move_word_backward(),
        
        // Enter insert mode
        KeyCode::Char('i') => self.vim_mode = VimMode::Insert,
        KeyCode::Char('a') => {
            self.editor_move_cursor_right();
            self.vim_mode = VimMode::Insert;
        }
        KeyCode::Char('I') => {
            self.editor_move_to_line_start();
            self.vim_mode = VimMode::Insert;
        }
        KeyCode::Char('A') => {
            self.editor_move_to_line_end();
            self.vim_mode = VimMode::Insert;
        }
        KeyCode::Char('o') => {
            self.editor_insert_line_below();
            self.vim_mode = VimMode::Insert;
        }
        KeyCode::Char('O') => {
            self.editor_insert_line_above();
            self.vim_mode = VimMode::Insert;
        }
        
        // Delete commands
        KeyCode::Char('x') => self.editor_delete_char(),
        KeyCode::Char('d') => self.editor_delete_line(), // simplified dd
        KeyCode::Char('D') => self.editor_delete_to_end(),
        
        _ => {}
    }
    Ok(())
}

fn handle_vim_insert_mode(&mut self, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            self.vim_mode = VimMode::Normal;
            // Move cursor back one (vim behavior)
            if self.editor_cursor.1 > 0 {
                self.editor_cursor.1 -= 1;
            }
        }
        // Standard insert mode keys (existing behavior)
        KeyCode::Enter => self.editor_insert_newline(),
        KeyCode::Backspace => self.editor_backspace(),
        KeyCode::Delete => self.editor_delete(),
        KeyCode::Left => self.editor_move_cursor_left(),
        KeyCode::Right => self.editor_move_cursor_right(),
        KeyCode::Up => self.editor_move_cursor_up(),
        KeyCode::Down => self.editor_move_cursor_down(),
        KeyCode::Char(c) => self.editor_insert_char(c),
        _ => {}
    }
    Ok(())
}
```

### 4. New Editor Methods

```rust
// New methods to add to App impl

fn editor_move_to_line_start(&mut self) {
    self.editor_cursor.1 = 0;
}

fn editor_move_to_line_end(&mut self) {
    if let Some(line) = self.editor_content.get(self.editor_cursor.0) {
        self.editor_cursor.1 = line.len();
    }
}

fn editor_move_to_first_line(&mut self) {
    self.editor_cursor = (0, 0);
    self.editor_scroll = 0;
}

fn editor_move_to_last_line(&mut self) {
    if !self.editor_content.is_empty() {
        self.editor_cursor.0 = self.editor_content.len() - 1;
        self.editor_cursor.1 = 0;
    }
}

fn editor_move_word_forward(&mut self) {
    let (line, col) = self.editor_cursor;
    if let Some(content) = self.editor_content.get(line) {
        let chars: Vec<char> = content.chars().collect();
        let mut new_col = col;
        
        // Skip current word
        while new_col < chars.len() && !chars[new_col].is_whitespace() {
            new_col += 1;
        }
        // Skip whitespace
        while new_col < chars.len() && chars[new_col].is_whitespace() {
            new_col += 1;
        }
        
        if new_col < chars.len() {
            self.editor_cursor.1 = new_col;
        } else if line + 1 < self.editor_content.len() {
            // Move to next line
            self.editor_cursor = (line + 1, 0);
        }
    }
}

fn editor_move_word_backward(&mut self) {
    let (line, col) = self.editor_cursor;
    if col > 0 {
        if let Some(content) = self.editor_content.get(line) {
            let chars: Vec<char> = content.chars().collect();
            let mut new_col = col.saturating_sub(1);
            
            // Skip whitespace
            while new_col > 0 && chars[new_col].is_whitespace() {
                new_col -= 1;
            }
            // Skip to start of word
            while new_col > 0 && !chars[new_col - 1].is_whitespace() {
                new_col -= 1;
            }
            
            self.editor_cursor.1 = new_col;
        }
    } else if line > 0 {
        // Move to end of previous line
        self.editor_cursor.0 = line - 1;
        self.editor_cursor.1 = self.editor_content
            .get(line - 1)
            .map_or(0, |l| l.len());
    }
}

fn editor_insert_line_below(&mut self) {
    let line = self.editor_cursor.0;
    self.editor_content.insert(line + 1, String::new());
    self.editor_cursor = (line + 1, 0);
}

fn editor_insert_line_above(&mut self) {
    let line = self.editor_cursor.0;
    self.editor_content.insert(line, String::new());
    self.editor_cursor = (line, 0);
}

fn editor_delete_char(&mut self) {
    // Delete character under cursor (like 'x')
    let (line, col) = self.editor_cursor;
    if let Some(content) = self.editor_content.get_mut(line) {
        if col < content.len() {
            content.remove(col);
        }
    }
}

fn editor_delete_line(&mut self) {
    // Delete entire line (like 'dd')
    if !self.editor_content.is_empty() {
        let line = self.editor_cursor.0;
        self.editor_content.remove(line);
        
        if self.editor_content.is_empty() {
            self.editor_content.push(String::new());
        }
        
        // Adjust cursor
        if self.editor_cursor.0 >= self.editor_content.len() {
            self.editor_cursor.0 = self.editor_content.len().saturating_sub(1);
        }
        self.editor_cursor.1 = 0;
    }
}

fn editor_delete_to_end(&mut self) {
    // Delete from cursor to end of line (like 'D')
    let (line, col) = self.editor_cursor;
    if let Some(content) = self.editor_content.get_mut(line) {
        content.truncate(col);
    }
}
```

### 5. UI Changes

#### Status Bar (`src/ui/status_bar.rs`)

```rust
// Update EDIT mode badge to show vim mode
AppMode::Editing => {
    let vim_label = match app.vim_mode {
        VimMode::Normal => "NORMAL",
        VimMode::Insert => "INSERT",
    };
    let (bg, fg) = match app.vim_mode {
        VimMode::Normal => (HackerTheme::MODE_NORMAL_BG, HackerTheme::MODE_NORMAL_FG),
        VimMode::Insert => (HackerTheme::MODE_EDIT_BG, HackerTheme::MODE_EDIT_FG),
    };
    spans.push(Span::styled(
        format!(" {} {} ", BoxChars::SCANNER, vim_label),
        Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
    ));
}
```

#### Editor (`src/ui/editor.rs`)

- Change cursor style based on vim mode (block for Normal, line for Insert)

---

## File Changes Summary

| File | Changes |
|------|---------|
| `src/app.rs` | Add `VimMode` enum, `vim_mode` field, refactor key handlers, add ~10 new editor methods |
| `src/ui/status_bar.rs` | Update mode display to show NORMAL/INSERT |
| `src/ui/editor.rs` | Optional: different cursor styles per mode |

**Estimated Lines of Code:** ~150-200 new lines

---

## Keybinding Reference

| Key | Mode | Action |
|-----|------|--------|
| `h` | Normal | Move left |
| `j` | Normal | Move down |
| `k` | Normal | Move up |
| `l` | Normal | Move right |
| `0` | Normal | Move to line start |
| `$` | Normal | Move to line end |
| `g` | Normal | Move to first line |
| `G` | Normal | Move to last line |
| `w` | Normal | Move word forward |
| `b` | Normal | Move word backward |
| `i` | Normal | Insert before cursor |
| `a` | Normal | Insert after cursor |
| `I` | Normal | Insert at line start |
| `A` | Normal | Insert at line end |
| `o` | Normal | Insert line below |
| `O` | Normal | Insert line above |
| `x` | Normal | Delete char under cursor |
| `d` | Normal | Delete line |
| `D` | Normal | Delete to end of line |
| `Esc` | Normal | Exit edit mode |
| `q` | Normal | Exit edit mode |
| `Esc` | Insert | Return to Normal mode |
| (typing) | Insert | Insert characters |

---

## Implementation Plan

1. **Phase 1**: Add `VimMode` enum and state (~15 min)
2. **Phase 2**: Implement Normal mode key handler (~30 min)
3. **Phase 3**: Implement Insert mode key handler (~15 min)
4. **Phase 4**: Add new editor methods (~45 min)
5. **Phase 5**: Update UI (status bar, cursor) (~20 min)
6. **Phase 6**: Testing and fixes (~30 min)

**Total Estimated Time:** ~2.5 hours

---

## Future Enhancements

After the minimal implementation is stable, consider adding:

1. **Undo/Redo** - Most requested feature after basic vim
2. **Visual Mode** - For selection and bulk operations
3. **Yank/Paste** - Clipboard operations with `y`, `p`, `P`
4. **Count Prefixes** - `5j`, `3dd`, etc.
5. **Text Objects** - `iw`, `aw`, `i"`, `a"`, etc.
6. **Search** - `/pattern`, `n`, `N`
