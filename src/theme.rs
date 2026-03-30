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
        Self {
            border_normal: Color::DarkGray,
            border_focused: accent,
            highlight: accent,
            text_normal: Color::White,
            text_dim: Color::DarkGray,
        }
    }

    /// Convert a color name to a Color value
    pub fn from_name(name: &str) -> Color {
        match name.to_lowercase().as_str() {
            "orange" => Color::Rgb(255, 165, 0),
            "blue" => Color::Blue,
            "green" => Color::Green,
            "red" => Color::Red,
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
        assert_eq!(theme.border_normal, Color::DarkGray);
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
