//! Modern Clean Theme
//!
//! A minimal, modern color palette inspired by popular code editors.

use ratatui::style::Color;

/// Modern theme colors - Clean and minimal
pub struct HackerTheme;

impl HackerTheme {
    // === PRIMARY COLORS ===
    /// Primary accent - soft blue
    pub const MATRIX_GREEN: Color = Color::Rgb(86, 156, 214);
    /// Bright accent for highlights
    pub const MATRIX_GREEN_BRIGHT: Color = Color::Rgb(106, 176, 234);
    /// Dim accent for less important elements
    pub const MATRIX_GREEN_DIM: Color = Color::Rgb(66, 136, 194);
    /// Dark accent for subtle elements
    pub const MATRIX_GREEN_DARK: Color = Color::Rgb(46, 116, 174);

    // === ACCENT COLORS ===
    /// Cyan for active/selected elements
    pub const CYBER_CYAN: Color = Color::Rgb(78, 201, 176);
    /// Dim cyan for secondary highlights
    pub const CYBER_CYAN_DIM: Color = Color::Rgb(58, 161, 146);
    /// Magenta/pink for special elements
    pub const NEON_PINK: Color = Color::Rgb(206, 145, 192);
    /// Purple for alternates
    pub const ELECTRIC_PURPLE: Color = Color::Rgb(180, 142, 212);
    /// Warning orange
    pub const AMBER_WARNING: Color = Color::Rgb(220, 165, 80);
    /// Error red
    pub const NEON_RED: Color = Color::Rgb(244, 108, 117);
    /// Success green
    pub const NEON_GREEN: Color = Color::Rgb(152, 195, 121);

    // === BACKGROUND/BASE COLORS ===
    /// Deep background
    pub const VOID_BLACK: Color = Color::Rgb(30, 30, 30);
    /// Slightly lighter background for panels
    pub const DARK_BG: Color = Color::Rgb(37, 37, 38);
    /// Surface color for elevated elements
    pub const SURFACE: Color = Color::Rgb(45, 45, 48);
    /// Border color (dim)
    pub const BORDER_DIM: Color = Color::Rgb(60, 60, 60);
    /// Border color (active)
    pub const BORDER_ACTIVE: Color = Color::Rgb(86, 156, 214);

    // === TEXT COLORS ===
    /// Primary text - bright for readability
    pub const TEXT_PRIMARY: Color = Color::Rgb(212, 212, 212);
    /// Secondary text
    pub const TEXT_SECONDARY: Color = Color::Rgb(156, 156, 156);
    /// Muted text for less important info
    pub const TEXT_MUTED: Color = Color::Rgb(100, 100, 100);
    /// Comment text
    pub const TEXT_COMMENT: Color = Color::Rgb(106, 153, 85);

    // === SYNTAX HIGHLIGHTING ===
    /// HTTP methods (GET, POST, etc.)
    pub const SYNTAX_METHOD: Color = Color::Rgb(86, 156, 214);
    /// URLs
    pub const SYNTAX_URL: Color = Color::Rgb(206, 145, 120);
    /// Headers
    pub const SYNTAX_HEADER: Color = Color::Rgb(156, 220, 254);
    /// Header values
    pub const SYNTAX_VALUE: Color = Color::Rgb(206, 145, 120);
    /// Section markers [Asserts], etc.
    pub const SYNTAX_SECTION: Color = Color::Rgb(220, 165, 80);
    /// Variables {{var}}
    pub const SYNTAX_VARIABLE: Color = Color::Rgb(78, 201, 176);
    /// Keywords
    pub const SYNTAX_KEYWORD: Color = Color::Rgb(197, 134, 192);
    /// JSON/data content
    pub const SYNTAX_DATA: Color = Color::Rgb(181, 206, 168);
    /// Status line
    pub const SYNTAX_STATUS: Color = Color::Rgb(86, 156, 214);

    // === STATUS CODES ===
    /// 2xx success
    pub const STATUS_2XX: Color = Color::Rgb(152, 195, 121);
    /// 3xx redirect
    pub const STATUS_3XX: Color = Color::Rgb(220, 165, 80);
    /// 4xx client error
    pub const STATUS_4XX: Color = Color::Rgb(244, 108, 117);
    /// 5xx server error
    pub const STATUS_5XX: Color = Color::Rgb(224, 88, 97);

    // === MODE BADGES ===
    pub const MODE_NORMAL_BG: Color = Color::Rgb(45, 55, 72);
    pub const MODE_NORMAL_FG: Color = Color::Rgb(86, 156, 214);
    pub const MODE_EDIT_BG: Color = Color::Rgb(72, 55, 45);
    pub const MODE_EDIT_FG: Color = Color::Rgb(220, 165, 80);
    pub const MODE_SEARCH_BG: Color = Color::Rgb(45, 65, 65);
    pub const MODE_SEARCH_FG: Color = Color::Rgb(78, 201, 176);
    pub const MODE_COMMAND_BG: Color = Color::Rgb(55, 45, 65);
    pub const MODE_COMMAND_FG: Color = Color::Rgb(180, 142, 212);
    pub const MODE_FILTER_BG: Color = Color::Rgb(65, 55, 45);
    pub const MODE_FILTER_FG: Color = Color::Rgb(220, 165, 80);

    // === ASSERTIONS ===
    pub const ASSERT_PASS: Color = Color::Rgb(152, 195, 121);
    pub const ASSERT_FAIL: Color = Color::Rgb(244, 108, 117);
    pub const ASSERT_PENDING: Color = Color::Rgb(100, 100, 100);

    // === SPECIAL EFFECTS ===
    /// Cursor color
    pub const CURSOR_BG: Color = Color::Rgb(86, 156, 214);
    pub const CURSOR_FG: Color = Color::Rgb(30, 30, 30);
    /// Selected item background
    pub const SELECTED_BG: Color = Color::Rgb(55, 55, 60);
    pub const SELECTED_FG: Color = Color::Rgb(212, 212, 212);
    /// Running indicator
    pub const RUNNING: Color = Color::Rgb(220, 165, 80);
}

/// Box drawing characters for borders
pub struct BoxChars;

impl BoxChars {
    // Standard box drawing
    pub const TOP_LEFT: &'static str = "┌";
    pub const TOP_RIGHT: &'static str = "┐";
    pub const BOTTOM_LEFT: &'static str = "└";
    pub const BOTTOM_RIGHT: &'static str = "┘";
    pub const HORIZONTAL: &'static str = "─";
    pub const VERTICAL: &'static str = "│";

    // Double line
    pub const DOUBLE_TOP_LEFT: &'static str = "╔";
    pub const DOUBLE_TOP_RIGHT: &'static str = "╗";
    pub const DOUBLE_BOTTOM_LEFT: &'static str = "╚";
    pub const DOUBLE_BOTTOM_RIGHT: &'static str = "╝";
    pub const DOUBLE_HORIZONTAL: &'static str = "═";
    pub const DOUBLE_VERTICAL: &'static str = "║";

    // Block characters for indicators
    pub const BLOCK_FULL: &'static str = "█";
    pub const BLOCK_LIGHT: &'static str = "░";
    pub const BLOCK_MEDIUM: &'static str = "▒";
    pub const BLOCK_DARK: &'static str = "▓";

    // Arrows and symbols - cleaner, minimal style
    pub const ARROW_RIGHT: &'static str = "›";
    pub const ARROW_DOWN: &'static str = "›";
    pub const BULLET: &'static str = "•";
    pub const DIAMOND: &'static str = "◇";
    pub const TRIANGLE_RIGHT: &'static str = "▸";
    pub const TRIANGLE_DOWN: &'static str = "▾";

    // Clean decorations
    pub const GLITCH_1: &'static str = "░";
    pub const GLITCH_2: &'static str = "▒";
    pub const SCANNER: &'static str = "▌";
    pub const TERMINAL_PROMPT: &'static str = "›";
    pub const LAMBDA: &'static str = "λ";
    pub const CHECK: &'static str = "✓";
    pub const CROSS: &'static str = "✕";
    pub const DOT: &'static str = "·";

    // Spinner frames for loading animation
    pub const SPINNER: [&'static str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    // Alternative spinner (blocks)
    pub const SPINNER_BLOCKS: [&'static str; 4] = ["▖", "▘", "▝", "▗"];

    // Simple spinner
    pub const SPINNER_MATRIX: [&'static str; 4] = ["◐", "◓", "◑", "◒"];

    // Loading bar
    pub const LOADING_BAR: [&'static str; 5] = ["[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]"];

    /// Get spinner frame by index
    pub fn spinner(frame: usize) -> &'static str {
        Self::SPINNER[frame % Self::SPINNER.len()]
    }

    /// Get matrix spinner frame by index
    pub fn spinner_matrix(frame: usize) -> &'static str {
        Self::SPINNER_MATRIX[frame % Self::SPINNER_MATRIX.len()]
    }

    /// Get loading bar frame by index
    pub fn loading_bar(frame: usize) -> &'static str {
        Self::LOADING_BAR[frame % Self::LOADING_BAR.len()]
    }
}
