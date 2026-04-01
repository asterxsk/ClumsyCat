use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub border_normal: Color,
    pub border_focused: Color,
    pub highlight: Color,
    pub text_normal: Color,
    pub text_dim: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::with_accent("orange")
    }
}

impl Theme {
    /// Create a theme with the specified accent color
    pub fn with_accent(accent_color: &str) -> Self {
        let accent = Self::from_name(accent_color);
        // Increase border thickness by using a brighter border color for both normal and focused
        Self {
            border_normal: Color::Rgb(100, 100, 100),
            border_focused: accent,
            highlight: accent,
            text_normal: Color::White,
            text_dim: Color::DarkGray,
        }
    }

    /// Create a theme with a custom hex color
    pub fn with_custom_hex(hex_color: &str) -> Self {
        let accent = Self::from_hex(hex_color);
        Self {
            border_normal: Color::Rgb(100, 100, 100),
            border_focused: accent,
            highlight: accent,
            text_normal: Color::White,
            text_dim: Color::DarkGray,
        }
    }

    /// Parse a hex color string to Color
    pub fn from_hex(hex: &str) -> Color {
        if hex.len() == 7 && hex.starts_with('#') {
            let hex = &hex[1..];
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        Color::Rgb(255, 165, 0) // Default to orange if parsing fails
    }

    /// Convert a color name to a Color value
    pub fn from_name(name: &str) -> Color {
        match name.to_lowercase().as_str() {
            "orange" => Color::Rgb(255, 165, 0),
            "red" => Color::Red,
            "purple" => Color::Rgb(128, 0, 128),
            "blue" => Color::Blue,
            "light_blue" => Color::Rgb(173, 216, 230),
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            _ => Color::Rgb(255, 165, 0), // Default to orange
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.text_normal, Color::White);
        assert_eq!(theme.text_dim, Color::DarkGray);
        assert_eq!(theme.border_normal, Color::Rgb(100, 100, 100));
    }

    #[test]
    fn test_theme_with_accent_orange() {
        let theme = Theme::with_accent("orange");
        assert_eq!(theme.border_focused, Color::Rgb(255, 165, 0));
        assert_eq!(theme.highlight, Color::Rgb(255, 165, 0));
    }

    #[test]
    fn test_theme_with_accent_blue() {
        let theme = Theme::with_accent("blue");
        assert_eq!(theme.border_focused, Color::Blue);
        assert_eq!(theme.highlight, Color::Blue);
    }

    #[test]
    fn test_theme_with_accent_green() {
        let theme = Theme::with_accent("green");
        assert_eq!(theme.border_focused, Color::Green);
        assert_eq!(theme.highlight, Color::Green);
    }

    #[test]
    fn test_theme_from_name_lowercase() {
        let color = Theme::from_name("blue");
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_theme_from_name_uppercase() {
        let color = Theme::from_name("BLUE");
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_theme_from_name_mixed_case() {
        let color = Theme::from_name("BlUe");
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_theme_from_name_unknown_defaults_to_orange() {
        let color = Theme::from_name("unknown");
        assert_eq!(color, Color::Rgb(255, 165, 0));
    }
}
