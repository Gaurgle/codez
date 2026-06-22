use ratatui::style::Color;

pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const OVERLAY: Color = Color::Rgb(127, 132, 156);
pub const SURFACE: Color = Color::Rgb(69, 71, 90);
pub const SELECTION: Color = Color::Rgb(49, 50, 68);
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);
pub const MAUVE: Color = Color::Rgb(203, 166, 247);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const BLUE: Color = Color::Rgb(137, 180, 250);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);
pub const RED: Color = Color::Rgb(243, 139, 168);
pub const PEACH: Color = Color::Rgb(250, 179, 135);

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

/// ANSI truecolor foreground escape for a `Color::Rgb`; empty for other variants.
pub fn ansi(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
        _ => String::new(),
    }
}

/// Distinct accent color per category, for the row tag.
pub fn category_color(key: &str) -> Color {
    match key {
        "http" => SAPPHIRE,
        "exit" => PEACH,
        "curl" => GREEN,
        "git" => MAUVE,
        "errno" => YELLOW,
        "ble" => BLUE,
        "rust" => RED,
        "docker" => LAVENDER,
        "podman" => MAUVE,
        _ => OVERLAY,
    }
}

/// Color for a group header, by its leading band/word.
pub fn group_color(group: &str) -> Color {
    if group.starts_with("2xx") || group.starts_with("Success") {
        GREEN
    } else if group.starts_with("3xx") {
        BLUE
    } else if group.starts_with("4xx") {
        YELLOW
    } else if group.starts_with("5xx") || group.starts_with("Error") {
        RED
    } else if group.starts_with("Signal") {
        PEACH
    } else {
        MAUVE
    }
}
