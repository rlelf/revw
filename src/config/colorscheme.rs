use ratatui::style::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorScheme {
    pub name: &'static str,
    pub background: Color,
    pub border: Color,
    pub text: Color,
    pub text_dim: Color,
    pub highlight: Color,
    pub selected: Color,
    pub key: Color,
    pub string: Color,
    pub number: Color,
    pub boolean: Color,
    pub bracket: Color,
}

impl ColorScheme {
    pub fn default() -> Self {
        Self {
            name: "Default",
            background: Color::Rgb(26, 28, 34),
            border: Color::DarkGray,
            text: Color::Gray,
            text_dim: Color::DarkGray,
            highlight: Color::Yellow,
            selected: Color::Cyan,
            key: Color::Rgb(156, 220, 254),      // Light blue
            string: Color::Rgb(206, 145, 120),   // Orange/peach
            number: Color::Rgb(181, 206, 168),   // Light green
            boolean: Color::Rgb(86, 156, 214),   // Purple/blue
            bracket: Color::Rgb(255, 217, 102),  // Yellow/gold
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "Monokai",
            background: Color::Rgb(39, 40, 34),
            border: Color::Rgb(73, 72, 62),
            text: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(117, 113, 94),
            highlight: Color::Rgb(255, 255, 0),
            selected: Color::Rgb(102, 217, 239),
            key: Color::Rgb(102, 217, 239),      // Cyan
            string: Color::Rgb(230, 219, 116),   // Yellow
            number: Color::Rgb(174, 129, 255),   // Purple
            boolean: Color::Rgb(174, 129, 255),  // Purple
            bracket: Color::Rgb(249, 38, 114),   // Pink
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized",
            background: Color::Rgb(0, 43, 54),
            border: Color::Rgb(7, 54, 66),
            text: Color::Rgb(131, 148, 150),
            text_dim: Color::Rgb(88, 110, 117),
            highlight: Color::Rgb(181, 137, 0),
            selected: Color::Rgb(42, 161, 152),
            key: Color::Rgb(38, 139, 210),       // Blue
            string: Color::Rgb(42, 161, 152),    // Cyan
            number: Color::Rgb(211, 54, 130),    // Magenta
            boolean: Color::Rgb(108, 113, 196),  // Violet
            bracket: Color::Rgb(203, 75, 22),    // Orange
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord",
            background: Color::Rgb(46, 52, 64),
            border: Color::Rgb(59, 66, 82),
            text: Color::Rgb(216, 222, 233),
            text_dim: Color::Rgb(76, 86, 106),
            highlight: Color::Rgb(235, 203, 139),
            selected: Color::Rgb(136, 192, 208),
            key: Color::Rgb(136, 192, 208),      // Frost cyan
            string: Color::Rgb(163, 190, 140),   // Green
            number: Color::Rgb(180, 142, 173),   // Purple
            boolean: Color::Rgb(129, 161, 193),  // Blue
            bracket: Color::Rgb(208, 135, 112),  // Orange
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::default()),
            "monokai" => Some(Self::monokai()),
            "solarized" | "solarized-dark" => Some(Self::solarized_dark()),
            "nord" => Some(Self::nord()),
            _ => None,
        }
    }

    pub fn all_schemes() -> Vec<Self> {
        vec![
            Self::default(),
            Self::monokai(),
            Self::solarized_dark(),
            Self::nord(),
        ]
    }
}
