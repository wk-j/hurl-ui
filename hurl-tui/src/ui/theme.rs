//! Hacker Theme
//!
//! Matrix-inspired cyberpunk color palette for the terminal hacker aesthetic.

use ratatui::style::Color;

/// Hacker theme colors - Matrix/Cyberpunk inspired
pub struct HackerTheme;

impl HackerTheme {
    // === PRIMARY COLORS ===
    /// Main matrix green - the signature hacker color
    pub const MATRIX_GREEN: Color = Color::Rgb(0, 255, 65);
    /// Bright matrix green for highlights
    pub const MATRIX_GREEN_BRIGHT: Color = Color::Rgb(57, 255, 20);
    /// Dim matrix green for less important elements
    pub const MATRIX_GREEN_DIM: Color = Color::Rgb(0, 180, 45);
    /// Dark green for subtle elements
    pub const MATRIX_GREEN_DARK: Color = Color::Rgb(0, 100, 25);

    // === ACCENT COLORS ===
    /// Cyan for active/selected elements - cyberpunk accent
    pub const CYBER_CYAN: Color = Color::Rgb(0, 255, 255);
    /// Dim cyan for secondary highlights
    pub const CYBER_CYAN_DIM: Color = Color::Rgb(0, 180, 180);
    /// Magenta/pink for special elements - neon accent
    pub const NEON_PINK: Color = Color::Rgb(255, 0, 128);
    /// Electric purple for alternates
    pub const ELECTRIC_PURPLE: Color = Color::Rgb(191, 0, 255);
    /// Warning orange
    pub const AMBER_WARNING: Color = Color::Rgb(255, 176, 0);
    /// Error red - but still neon
    pub const NEON_RED: Color = Color::Rgb(255, 0, 60);
    /// Success green - brighter variant
    pub const NEON_GREEN: Color = Color::Rgb(0, 255, 128);

    // === BACKGROUND/BASE COLORS ===
    /// Deep black background
    pub const VOID_BLACK: Color = Color::Rgb(0, 0, 0);
    /// Slightly lighter background for panels
    pub const DARK_BG: Color = Color::Rgb(10, 10, 15);
    /// Surface color for elevated elements
    pub const SURFACE: Color = Color::Rgb(20, 22, 28);
    /// Border color (dim)
    pub const BORDER_DIM: Color = Color::Rgb(30, 40, 35);
    /// Border color (active)
    pub const BORDER_ACTIVE: Color = Color::Rgb(0, 200, 50);

    // === TEXT COLORS ===
    /// Primary text - bright for readability
    pub const TEXT_PRIMARY: Color = Color::Rgb(200, 255, 200);
    /// Secondary text
    pub const TEXT_SECONDARY: Color = Color::Rgb(120, 160, 130);
    /// Muted text for less important info
    pub const TEXT_MUTED: Color = Color::Rgb(70, 90, 75);
    /// Comment text
    pub const TEXT_COMMENT: Color = Color::Rgb(80, 120, 90);

    // === SYNTAX HIGHLIGHTING ===
    /// HTTP methods (GET, POST, etc.)
    pub const SYNTAX_METHOD: Color = Color::Rgb(0, 255, 128);
    /// URLs
    pub const SYNTAX_URL: Color = Color::Rgb(0, 220, 255);
    /// Headers
    pub const SYNTAX_HEADER: Color = Color::Rgb(150, 255, 150);
    /// Header values
    pub const SYNTAX_VALUE: Color = Color::Rgb(180, 220, 180);
    /// Section markers [Asserts], etc.
    pub const SYNTAX_SECTION: Color = Color::Rgb(255, 176, 0);
    /// Variables {{var}}
    pub const SYNTAX_VARIABLE: Color = Color::Rgb(255, 100, 255);
    /// Keywords
    pub const SYNTAX_KEYWORD: Color = Color::Rgb(0, 255, 200);
    /// JSON/data content
    pub const SYNTAX_DATA: Color = Color::Rgb(180, 255, 180);
    /// Status line
    pub const SYNTAX_STATUS: Color = Color::Rgb(200, 100, 255);

    // === STATUS CODES ===
    /// 2xx success
    pub const STATUS_2XX: Color = Color::Rgb(0, 255, 128);
    /// 3xx redirect
    pub const STATUS_3XX: Color = Color::Rgb(255, 200, 0);
    /// 4xx client error
    pub const STATUS_4XX: Color = Color::Rgb(255, 100, 50);
    /// 5xx server error
    pub const STATUS_5XX: Color = Color::Rgb(255, 0, 80);

    // === MODE BADGES ===
    pub const MODE_NORMAL_BG: Color = Color::Rgb(0, 80, 40);
    pub const MODE_NORMAL_FG: Color = Color::Rgb(0, 255, 65);
    pub const MODE_EDIT_BG: Color = Color::Rgb(80, 60, 0);
    pub const MODE_EDIT_FG: Color = Color::Rgb(255, 200, 0);
    pub const MODE_SEARCH_BG: Color = Color::Rgb(0, 60, 80);
    pub const MODE_SEARCH_FG: Color = Color::Rgb(0, 255, 255);
    pub const MODE_COMMAND_BG: Color = Color::Rgb(60, 0, 80);
    pub const MODE_COMMAND_FG: Color = Color::Rgb(200, 100, 255);
    pub const MODE_FILTER_BG: Color = Color::Rgb(80, 40, 0);
    pub const MODE_FILTER_FG: Color = Color::Rgb(255, 150, 0);

    // === ASSERTIONS ===
    pub const ASSERT_PASS: Color = Color::Rgb(0, 255, 128);
    pub const ASSERT_FAIL: Color = Color::Rgb(255, 50, 80);
    pub const ASSERT_PENDING: Color = Color::Rgb(100, 140, 110);

    // === SPECIAL EFFECTS ===
    /// Cursor color
    pub const CURSOR_BG: Color = Color::Rgb(0, 255, 65);
    pub const CURSOR_FG: Color = Color::Rgb(0, 0, 0);
    /// Selected item background
    pub const SELECTED_BG: Color = Color::Rgb(0, 60, 30);
    pub const SELECTED_FG: Color = Color::Rgb(0, 255, 65);
    /// Running indicator
    pub const RUNNING: Color = Color::Rgb(255, 200, 0);
}

/// Box drawing characters for custom borders
pub struct BoxChars;

impl BoxChars {
    // Standard box drawing
    pub const TOP_LEFT: &'static str = "┌";
    pub const TOP_RIGHT: &'static str = "┐";
    pub const BOTTOM_LEFT: &'static str = "└";
    pub const BOTTOM_RIGHT: &'static str = "┘";
    pub const HORIZONTAL: &'static str = "─";
    pub const VERTICAL: &'static str = "│";

    // Double line (more "hacker" feel)
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

    // Arrows and symbols
    pub const ARROW_RIGHT: &'static str = "▶";
    pub const ARROW_DOWN: &'static str = "▼";
    pub const BULLET: &'static str = "●";
    pub const DIAMOND: &'static str = "◆";
    pub const TRIANGLE_RIGHT: &'static str = "►";
    pub const TRIANGLE_DOWN: &'static str = "▾";

    // Hacker-style decorations
    pub const GLITCH_1: &'static str = "░";
    pub const GLITCH_2: &'static str = "▒";
    pub const SCANNER: &'static str = "▌";
    pub const TERMINAL_PROMPT: &'static str = "❯";
    pub const LAMBDA: &'static str = "λ";
    pub const CHECK: &'static str = "✓";
    pub const CROSS: &'static str = "✗";
    pub const DOT: &'static str = "·";
}
