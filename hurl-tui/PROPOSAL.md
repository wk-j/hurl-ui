# Hurl TUI - Technical Proposal

**Project:** Terminal User Interface for Hurl HTTP Testing Tool  
**Version:** 1.0  
**Date:** January 2026  
**Status:** Draft

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Problem Statement](#problem-statement)
3. [Proposed Solution](#proposed-solution)
4. [Technical Architecture](#technical-architecture)
5. [Technology Stack](#technology-stack)
6. [Feature Specification](#feature-specification)
7. [User Interface Design](#user-interface-design)
8. [Project Structure](#project-structure)
9. [Implementation Plan](#implementation-plan)
10. [Risk Assessment](#risk-assessment)
11. [Success Criteria](#success-criteria)
12. [Future Considerations](#future-considerations)

---

## 1. Executive Summary

This proposal outlines the development of **Hurl TUI**, a terminal-based user interface for [Hurl](https://hurl.dev/), the command-line tool for running HTTP requests defined in plain text format. Built with Rust and the Ratatui framework, Hurl TUI will provide an interactive, keyboard-driven experience for creating, editing, running, and debugging Hurl files directly in the terminal.

### Key Benefits

- **Improved Developer Experience**: Visual feedback without leaving the terminal
- **Faster Iteration**: Edit and run requests in a single interface
- **Better Debugging**: Clear assertion results and response inspection
- **Cross-Platform**: Works on Linux, macOS, and Windows

---

## 2. Problem Statement

### Current Workflow Limitations

While Hurl is a powerful CLI tool, developers face several challenges in their daily workflow:

| Challenge | Impact |
|-----------|--------|
| **Context Switching** | Developers must switch between text editor and terminal to edit and run requests |
| **Limited Visibility** | Raw CLI output lacks structure for complex responses |
| **No Visual Feedback** | Assertion results require parsing text output |
| **Manual File Navigation** | Finding and organizing `.hurl` files requires separate tooling |
| **Environment Management** | Switching between environments requires manual variable handling |

### Target Users

1. **API Developers** - Building and testing HTTP endpoints
2. **QA Engineers** - Writing and maintaining API test suites
3. **DevOps Engineers** - Automating HTTP-based health checks
4. **Backend Developers** - Debugging API integrations

---

## 3. Proposed Solution

### Overview

Hurl TUI is an interactive terminal application that combines file browsing, editing, request execution, and response visualization in a single, unified interface.

### Core Capabilities

```
+------------------+     +------------------+     +------------------+
|   File Browser   | --> |   Hurl Editor    | --> |  Request Runner  |
|                  |     |                  |     |                  |
| - Tree view      |     | - Syntax HL      |     | - Execute hurl   |
| - Search/filter  |     | - Edit in-place  |     | - Show progress  |
| - CRUD ops       |     | - Autocomplete   |     | - Capture timing |
+------------------+     +------------------+     +------------------+
                                                           |
                                                           v
+------------------+     +------------------+     +------------------+
|    Variables     | <-- |   Assertions     | <-- | Response Viewer  |
|                  |     |                  |     |                  |
| - Env switching  |     | - Pass/fail      |     | - Pretty print   |
| - Secret mask    |     | - Expected/actual|     | - Headers view   |
| - Edit values    |     | - Jump to source |     | - Timing info    |
+------------------+     +------------------+     +------------------+
```

---

## 4. Technical Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           HURL TUI                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Presentation Layer                        │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐    │   │
│  │  │  Layout   │ │  Widgets  │ │  Themes   │ │  Keymaps  │    │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Application Layer                         │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐    │   │
│  │  │App State  │ │  Actions  │ │  Events   │ │  Effects  │    │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      Core Layer                              │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐    │   │
│  │  │  Parser   │ │  Runner   │ │  Config   │ │   Files   │    │   │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────┘    │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                               │                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    External Layer                            │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐                  │   │
│  │  │   Hurl    │ │ File Sys  │ │  Network  │                  │   │
│  │  │  Binary   │ │           │ │           │                  │   │
│  │  └───────────┘ └───────────┘ └───────────┘                  │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Design Patterns

1. **Elm Architecture (TEA)**: Unidirectional data flow with Model-Update-View pattern
2. **Command Pattern**: Encapsulated actions for undo/redo capability
3. **Observer Pattern**: Event-driven updates for async operations
4. **Strategy Pattern**: Pluggable formatters for response rendering

### State Management

```rust
// Simplified state model
struct AppState {
    // UI State
    active_panel: Panel,
    mode: AppMode,
    
    // Data State
    file_tree: FileTree,
    current_file: Option<HurlFile>,
    execution_result: Option<ExecutionResult>,
    
    // Configuration
    config: Config,
    environment: Environment,
}

enum Message {
    // Navigation
    NavigateUp,
    NavigateDown,
    SelectPanel(Panel),
    
    // Actions
    OpenFile(PathBuf),
    RunRequest,
    SaveFile,
    
    // Results
    RequestCompleted(ExecutionResult),
    RequestFailed(Error),
}
```

---

## 5. Technology Stack

### Core Dependencies

| Component | Library | Version | Purpose |
|-----------|---------|---------|---------|
| Language | Rust | 1.75+ | Systems programming, memory safety |
| TUI Framework | Ratatui | 0.28 | Terminal UI rendering |
| Terminal Backend | Crossterm | 0.28 | Cross-platform terminal manipulation |
| Async Runtime | Tokio | 1.x | Async I/O, process spawning |
| Serialization | Serde | 1.x | JSON/TOML parsing |
| Syntax Highlighting | Syntect | 5.x | Code highlighting |
| Text Editing | tui-textarea | 0.6 | Text area widget |

### Why Rust?

1. **Alignment with Hurl**: Hurl is written in Rust, enabling potential deep integration
2. **Performance**: Zero-cost abstractions, no GC pauses
3. **Safety**: Memory safety guarantees prevent crashes
4. **Ecosystem**: Excellent TUI libraries (Ratatui)
5. **Cross-Platform**: Single codebase for all platforms

### Why Ratatui?

1. **Active Development**: Fork of tui-rs with active maintenance
2. **Widget Library**: Rich set of built-in widgets
3. **Flexibility**: Custom widget support
4. **Documentation**: Comprehensive examples and docs
5. **Community**: Growing ecosystem of extensions

---

## 6. Feature Specification

### Phase 1: Core Features (MVP)

#### F1.1 File Browser
- Display `.hurl` files in tree structure
- Expand/collapse directories
- Navigate with keyboard (j/k, arrows)
- Quick search/filter by name

#### F1.2 Hurl File Viewer
- Display file contents with line numbers
- Syntax highlighting for Hurl format
- Scroll navigation

#### F1.3 Request Execution
- Run current file using Hurl CLI
- Display execution status
- Show timing information

#### F1.4 Response Viewer
- Display response body
- JSON/XML pretty printing
- Show headers
- Display status code

#### F1.5 Assertions Panel
- List all assertions
- Show pass/fail status with icons
- Display expected vs actual on failure

### Phase 2: Enhanced Features

#### F2.1 In-Place Editing
- Edit Hurl files directly
- Vim-style keybindings
- Auto-save option

#### F2.2 Environment Management
- Switch between environments
- View/edit variables
- Secret value masking

#### F2.3 Request History
- Store executed requests
- Re-run from history
- Compare responses

#### F2.4 Configuration
- Customizable keybindings
- Theme support
- Layout preferences

### Phase 3: Advanced Features

#### F3.1 Collections
- Organize requests into collections
- Run collection as suite
- Generate reports

#### F3.2 Export
- Export to cURL
- Generate code snippets
- Export to Postman format

#### F3.3 Collaboration
- Share collections
- Import from URLs
- Git integration

---

## 7. User Interface Design

### Layout Specification

```
┌─ Files ─────────┬─ Editor ──────────────────────────────────────────┐
│                 │                                                    │
│ Height: 100%    │ Height: 60%                                        │
│ Width: 20%      │ Width: 80%                                         │
│                 │                                                    │
│ - Tree view     │ - Line numbers                                     │
│ - Icons         │ - Syntax highlighting                              │
│ - Selection     │ - Cursor (edit mode)                               │
│                 │                                                    │
├─────────────────┼─────────────────────────┬────────────────────────-─┤
│ Variables       │ Response                │ Assertions               │
│                 │                         │                          │
│ Height: 30%     │ Height: 40%             │ Height: 40%              │
│ Width: 20%      │ Width: 50%              │ Width: 30%               │
│                 │                         │                          │
│ - Key-value     │ - Body (formatted)      │ - Status icons           │
│ - Env name      │ - Headers tab           │ - Expected/actual        │
│ - Edit support  │ - Timing tab            │ - Source link            │
│                 │                         │                          │
└─────────────────┴─────────────────────────┴──────────────────────────┘
 [r]un [e]dit [v]ars [h]istory [?]help [q]uit     Status: Ready  ⏱ --ms
```

### Visual Mockup

```
┌─ Files ─────────┬─ api/users.hurl ─────────────────────────────────-─┐
│ 📁 api/         │  1 │ # Get all users                               │
│   ├─ users.hurl │  2 │ GET https://api.example.com/users             │
│   ├─ auth.hurl  │  3 │ Accept: application/json                      │
│   └─ orders.hurl│  4 │ Authorization: Bearer {{token}}               │
│ 📁 tests/       │  5 │                                               │
│   └─ smoke.hurl │  6 │ HTTP 200                                      │
│                 │  7 │ [Asserts]                                     │
│                 │  8 │ jsonpath "$.users" count > 0                  │
│                 │  9 │ jsonpath "$.users[0].id" exists               │
│                 │ 10 │                                               │
├─ Variables ─────┼─ Response ──────────────┬─ Assertions ─────────────┤
│ env: production │ Status: 200 OK          │ ✓ status == 200          │
│                 │ Time: 145ms             │ ✓ jsonpath count > 0     │
│ token: eyJhb... │                         │ ✓ jsonpath exists        │
│ base_url: https │ {                       │                          │
│ timeout: 30     │   "users": [            │ 3/3 passed               │
│                 │     {                   │                          │
│ [E] Switch Env  │       "id": 1,          │                          │
│                 │       "name": "Alice"   │                          │
│                 │     }                   │                          │
│                 │   ]                     │                          │
│                 │ }                       │                          │
└─────────────────┴─────────────────────────┴──────────────────────────┘
 [r]un [e]dit [v]ars [E]nv [h]ist [?]help [q]uit   ✓ Ready    ⏱ 145ms
```

### Keyboard Shortcuts

#### Global

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `?` | Toggle help overlay |
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `Ctrl+c` | Force quit |

#### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Move left / collapse |
| `l` / `→` | Move right / expand |
| `g` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |

#### Actions

| Key | Action |
|-----|--------|
| `Enter` | Select / Open / Run |
| `r` | Run request |
| `e` | Enter edit mode |
| `Esc` | Exit edit mode / Cancel |
| `v` | Toggle variables panel |
| `E` | Cycle environment |
| `/` | Search |
| `:` | Command mode |

#### Edit Mode

| Key | Action |
|-----|--------|
| `Esc` | Exit edit mode |
| Standard keys | Text editing |

#### Command Mode

| Command | Action |
|---------|--------|
| `:w` | Save file |
| `:q` | Quit |
| `:wq` | Save and quit |
| `:e <file>` | Open file |
| `:run` | Run current request |

---

## 8. Project Structure

```
hurl-tui/
├── Cargo.toml                 # Project manifest
├── Cargo.lock                 # Dependency lock file
├── README.md                  # Project documentation
├── LICENSE                    # MIT License
├── PROPOSAL.md                # This document
│
├── src/
│   ├── main.rs                # Entry point, terminal setup
│   ├── app.rs                 # Application state & logic
│   │
│   ├── ui/
│   │   ├── mod.rs             # UI module exports
│   │   ├── layout.rs          # Panel layout management
│   │   ├── file_browser.rs    # File tree widget
│   │   ├── editor.rs          # Hurl file editor widget
│   │   ├── response.rs        # Response viewer widget
│   │   ├── assertions.rs      # Assertions panel widget
│   │   ├── variables.rs       # Variables panel widget
│   │   ├── status_bar.rs      # Bottom status bar
│   │   └── help.rs            # Help overlay
│   │
│   ├── runner/
│   │   ├── mod.rs             # Runner module exports
│   │   └── executor.rs        # Hurl execution wrapper
│   │
│   ├── parser/
│   │   ├── mod.rs             # Parser module exports
│   │   └── hurl.rs            # Hurl file parser
│   │
│   ├── config/
│   │   ├── mod.rs             # Config module exports
│   │   ├── settings.rs        # Application settings
│   │   └── keymap.rs          # Keybinding configuration
│   │
│   └── events/
│       ├── mod.rs             # Events module exports
│       └── handler.rs         # Input event handling
│
├── config/
│   └── default.toml           # Default configuration
│
└── tests/
    ├── integration/           # Integration tests
    └── fixtures/              # Test fixtures (.hurl files)
```

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Application bootstrap, terminal setup/restore |
| `app.rs` | Central state management, message dispatch |
| `ui/*` | Widget rendering, layout calculation |
| `runner/*` | Hurl CLI invocation, output parsing |
| `parser/*` | Hurl file parsing, syntax analysis |
| `config/*` | Configuration loading, validation |
| `events/*` | Input event capture, debouncing |

---

## 9. Implementation Plan

### Timeline Overview

```
Week 1-2     Week 3-4     Week 5-6     Week 7-8     Week 9-10    Week 11-12
   │            │            │            │            │            │
   ▼            ▼            ▼            ▼            ▼            ▼
┌──────┐    ┌──────┐    ┌──────┐    ┌──────┐    ┌──────┐    ┌──────┐
│Found-│    │ Core │    │ UI   │    │ Edit │    │ Adv  │    │Polish│
│ation │    │ Feat │    │Polish│    │ Mode │    │ Feat │    │ Docs │
└──────┘    └──────┘    └──────┘    └──────┘    └──────┘    └──────┘
```

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic application structure and rendering

- [ ] Project setup with Cargo
- [ ] Terminal initialization with Crossterm
- [ ] Basic Ratatui layout
- [ ] Event loop implementation
- [ ] File tree loading and display
- [ ] Basic keyboard navigation

**Deliverable**: Application that displays file tree and responds to navigation

### Phase 2: Core Features (Weeks 3-4)

**Goal**: Request execution and response viewing

- [ ] Hurl file parsing
- [ ] Hurl CLI integration
- [ ] Response capture and display
- [ ] Assertion result parsing
- [ ] Status bar with timing
- [ ] Error handling

**Deliverable**: Functional request runner with response display

### Phase 3: UI Polish (Weeks 5-6)

**Goal**: Enhanced visual experience

- [ ] Syntax highlighting
- [ ] JSON/XML formatting
- [ ] Scrollable panels
- [ ] Panel resizing
- [ ] Color themes
- [ ] Help overlay

**Deliverable**: Polished, visually appealing interface

### Phase 4: Edit Mode (Weeks 7-8)

**Goal**: In-place file editing

- [ ] Text input handling
- [ ] Cursor management
- [ ] Line editing operations
- [ ] Save functionality
- [ ] Undo/redo (basic)
- [ ] Auto-completion (basic)

**Deliverable**: Full editing capability

### Phase 5: Advanced Features (Weeks 9-10)

**Goal**: Environment and history management

- [ ] Environment file loading
- [ ] Variable management UI
- [ ] Request history storage
- [ ] History navigation
- [ ] Configuration system
- [ ] Custom keybindings

**Deliverable**: Complete environment and history support

### Phase 6: Polish & Documentation (Weeks 11-12)

**Goal**: Production readiness

- [ ] Comprehensive testing
- [ ] Performance optimization
- [ ] Documentation
- [ ] Installation guides
- [ ] Example files
- [ ] Release packaging

**Deliverable**: v1.0 release

---

## 10. Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Hurl CLI output format changes | Medium | High | Abstract parser, version pinning |
| Terminal compatibility issues | Low | Medium | Extensive testing, Crossterm abstraction |
| Performance with large files | Low | Medium | Lazy loading, virtualized lists |
| Complex state management | Medium | Medium | Elm architecture, comprehensive tests |

### Project Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Scope creep | High | Medium | Strict MVP definition, phased approach |
| Dependency vulnerabilities | Low | High | Regular audits, minimal dependencies |
| Platform-specific bugs | Medium | Low | CI testing on all platforms |

### Mitigation Strategies

1. **Abstraction Layers**: Isolate external dependencies behind interfaces
2. **Feature Flags**: Enable/disable features without code changes
3. **Comprehensive Testing**: Unit, integration, and manual testing
4. **Incremental Releases**: Ship early, iterate based on feedback

---

## 11. Success Criteria

### Functional Requirements

- [ ] Browse and open `.hurl` files from file tree
- [ ] Execute Hurl requests and display results
- [ ] Show response body with formatting
- [ ] Display assertion results with pass/fail status
- [ ] Edit files in-place
- [ ] Manage environment variables
- [ ] Navigate entirely with keyboard

### Performance Requirements

| Metric | Target |
|--------|--------|
| Startup time | < 100ms |
| Frame render time | < 16ms (60 FPS) |
| File open time | < 50ms for files < 1MB |
| Memory usage | < 50MB baseline |

### Quality Requirements

- [ ] No crashes on normal usage
- [ ] Graceful error handling
- [ ] Clear error messages
- [ ] Consistent keyboard shortcuts
- [ ] Works on Linux, macOS, Windows

### User Experience Requirements

- [ ] Intuitive navigation
- [ ] Discoverable features (help overlay)
- [ ] Visual feedback for all actions
- [ ] Responsive interface (no blocking)

---

## 12. Future Considerations

### Potential Enhancements (Post v1.0)

1. **Deep Hurl Integration**
   - Link against Hurl library instead of CLI
   - Real-time validation as you type
   - Intelligent auto-completion

2. **Collaboration Features**
   - Share collections via URL
   - Team workspaces
   - Comments on requests

3. **Advanced Testing**
   - Test suite management
   - CI/CD integration
   - Report generation

4. **Protocol Support**
   - WebSocket testing
   - GraphQL support
   - gRPC support

5. **AI Features**
   - Request generation from description
   - Response analysis
   - Test suggestion

### Maintenance Plan

- Monthly dependency updates
- Quarterly feature releases
- Security patches within 48 hours
- Community issue triage weekly

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Hurl** | HTTP testing tool using plain text format |
| **TUI** | Terminal User Interface |
| **Ratatui** | Rust library for building TUIs |
| **Assertion** | Validation rule for HTTP responses |
| **Environment** | Set of variables for different contexts |

## Appendix B: References

- [Hurl Documentation](https://hurl.dev/)
- [Ratatui Documentation](https://ratatui.rs/)
- [Crossterm Documentation](https://docs.rs/crossterm/)
- [The Elm Architecture](https://guide.elm-lang.org/architecture/)

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | January 2026 | - | Initial proposal |

---

*End of Proposal*
