use ratatui::style::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorScheme {
    pub name: &'static str,
    pub background: Color,                  // Background color for cards and UI
    pub border: Color,                       // Default border color (fallback)
    pub window_border: Color,                // File window border color
    pub window_title: Color,                 // File window title (filename) color
    pub explorer_border: Color,              // Explorer window border color
    pub explorer_title: Color,               // Explorer window title (folder name) color
    pub card_border: Color,                  // Card border color (non-selected)
    pub text: Color,                         // Main text color
    pub text_dim: Color,                     // Dimmed text color (e.g., line numbers)
    pub highlight: Color,                    // Highlight color for search results
    pub selected: Color,                     // General selection color
    pub card_selected: Color,                // Border color for selected card
    pub card_visual: Color,                  // Border color for Visual mode selection
    pub card_title: Color,                   // Card title color (name, url, date, percentage)
    pub card_content: Color,                 // Card content text color (context)
    pub overlay_field_active: Color,         // Overlay field color when editing (Insert/Edit mode)
    pub overlay_field_selected: Color,       // Overlay field color when selected (Normal mode)
    pub overlay_field_placeholder: Color,    // Overlay field placeholder text color
    pub overlay_field_normal: Color,         // Overlay field normal text color
    pub explorer_folder: Color,              // Explorer folder name color
    pub explorer_file: Color,                // Explorer file name color
    pub explorer_file_selected: Color,       // Explorer selected file/folder color
    pub status_bar: Color,                   // Status bar text color
    pub key: Color,                          // JSON key color (Edit mode)
    pub string: Color,                       // JSON string value color (Edit mode)
    pub number: Color,                       // JSON number value color (Edit mode)
    pub boolean: Color,                      // JSON boolean/null value color (Edit mode)
    pub bracket: Color,                      // JSON bracket color (Edit mode)
}

impl ColorScheme {
    pub fn default() -> Self {
        Self {
            name: "Default",
            background: Color::Rgb(26, 28, 34),
            border: Color::DarkGray,
            window_border: Color::DarkGray,
            window_title: Color::Rgb(156, 220, 254),       // Light blue
            explorer_border: Color::DarkGray,
            explorer_title: Color::Cyan,
            card_border: Color::DarkGray,
            text: Color::Gray,
            text_dim: Color::DarkGray,
            highlight: Color::Yellow,
            selected: Color::Cyan,
            card_selected: Color::Yellow,
            card_visual: Color::Cyan,
            card_title: Color::Rgb(156, 220, 254),       // Light blue
            card_content: Color::Gray,
            overlay_field_active: Color::Yellow,         // Insert/Edit mode
            overlay_field_selected: Color::Cyan,         // Normal mode selection
            overlay_field_placeholder: Color::DarkGray,
            overlay_field_normal: Color::Gray,
            explorer_folder: Color::Cyan,
            explorer_file: Color::Rgb(180, 180, 180),  // Lighter gray
            explorer_file_selected: Color::White,       // White for selection
            status_bar: Color::Cyan,
            key: Color::Rgb(156, 220, 254),              // Light blue
            string: Color::Rgb(206, 145, 120),           // Orange/peach
            number: Color::Rgb(181, 206, 168),           // Light green
            boolean: Color::Rgb(86, 156, 214),           // Purple/blue
            bracket: Color::Rgb(255, 217, 102),          // Yellow/gold
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "Monokai",
            background: Color::Rgb(39, 40, 34),
            border: Color::Rgb(73, 72, 62),
            window_border: Color::Rgb(73, 72, 62),
            window_title: Color::Rgb(102, 217, 239),       // Cyan
            explorer_border: Color::Rgb(73, 72, 62),
            explorer_title: Color::Rgb(166, 226, 46),      // Green
            card_border: Color::Rgb(73, 72, 62),
            text: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(117, 113, 94),
            highlight: Color::Rgb(255, 255, 0),
            selected: Color::Rgb(102, 217, 239),
            card_selected: Color::Rgb(230, 219, 116),    // Yellow
            card_visual: Color::Rgb(102, 217, 239),      // Cyan
            card_title: Color::Rgb(102, 217, 239),       // Cyan
            card_content: Color::Rgb(248, 248, 242),
            overlay_field_active: Color::Rgb(230, 219, 116),     // Yellow - Insert/Edit mode
            overlay_field_selected: Color::Rgb(102, 217, 239),   // Cyan - Normal mode
            overlay_field_placeholder: Color::Rgb(117, 113, 94), // Dim
            overlay_field_normal: Color::Rgb(248, 248, 242),
            explorer_folder: Color::Rgb(102, 217, 239),          // Cyan
            explorer_file: Color::Rgb(200, 200, 194),            // Dimmer white
            explorer_file_selected: Color::Rgb(248, 248, 242),   // Bright white for selection
            status_bar: Color::Rgb(166, 226, 46),                // Green
            key: Color::Rgb(102, 217, 239),              // Cyan
            string: Color::Rgb(230, 219, 116),           // Yellow
            number: Color::Rgb(174, 129, 255),           // Purple
            boolean: Color::Rgb(174, 129, 255),          // Purple
            bracket: Color::Rgb(249, 38, 114),           // Pink
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized",
            background: Color::Rgb(0, 43, 54),
            border: Color::Rgb(7, 54, 66),
            window_border: Color::Rgb(7, 54, 66),
            window_title: Color::Rgb(38, 139, 210),        // Blue
            explorer_border: Color::Rgb(7, 54, 66),
            explorer_title: Color::Rgb(42, 161, 152),      // Cyan
            card_border: Color::Rgb(7, 54, 66),
            text: Color::Rgb(131, 148, 150),
            text_dim: Color::Rgb(88, 110, 117),
            highlight: Color::Rgb(181, 137, 0),
            selected: Color::Rgb(42, 161, 152),
            card_selected: Color::Rgb(181, 137, 0),      // Yellow
            card_visual: Color::Rgb(42, 161, 152),       // Cyan
            card_title: Color::Rgb(38, 139, 210),        // Blue
            card_content: Color::Rgb(131, 148, 150),
            overlay_field_active: Color::Rgb(181, 137, 0),       // Yellow - Insert/Edit mode
            overlay_field_selected: Color::Rgb(42, 161, 152),    // Cyan - Normal mode
            overlay_field_placeholder: Color::Rgb(88, 110, 117), // Dim
            overlay_field_normal: Color::Rgb(131, 148, 150),
            explorer_folder: Color::Rgb(38, 139, 210),           // Blue
            explorer_file: Color::Rgb(147, 161, 161),            // Base1
            explorer_file_selected: Color::Rgb(238, 232, 213),   // Base2 for selection
            status_bar: Color::Rgb(42, 161, 152),                // Cyan
            key: Color::Rgb(38, 139, 210),               // Blue
            string: Color::Rgb(42, 161, 152),            // Cyan
            number: Color::Rgb(211, 54, 130),            // Magenta
            boolean: Color::Rgb(108, 113, 196),          // Violet
            bracket: Color::Rgb(203, 75, 22),            // Orange
        }
    }

    pub fn nord() -> Self {
        Self {
            name: "Nord",
            background: Color::Rgb(46, 52, 64),
            border: Color::Rgb(59, 66, 82),
            window_border: Color::Rgb(59, 66, 82),
            window_title: Color::Rgb(136, 192, 208),       // Frost cyan
            explorer_border: Color::Rgb(59, 66, 82),
            explorer_title: Color::Rgb(163, 190, 140),     // Green
            card_border: Color::Rgb(59, 66, 82),
            text: Color::Rgb(216, 222, 233),
            text_dim: Color::Rgb(76, 86, 106),
            highlight: Color::Rgb(235, 203, 139),
            selected: Color::Rgb(136, 192, 208),
            card_selected: Color::Rgb(235, 203, 139),    // Yellow
            card_visual: Color::Rgb(136, 192, 208),      // Frost cyan
            card_title: Color::Rgb(136, 192, 208),       // Frost cyan
            card_content: Color::Rgb(216, 222, 233),
            overlay_field_active: Color::Rgb(235, 203, 139),     // Yellow - Insert/Edit mode
            overlay_field_selected: Color::Rgb(136, 192, 208),   // Frost cyan - Normal mode
            overlay_field_placeholder: Color::Rgb(76, 86, 106),  // Dim
            overlay_field_normal: Color::Rgb(216, 222, 233),
            explorer_folder: Color::Rgb(136, 192, 208),          // Frost cyan
            explorer_file: Color::Rgb(229, 233, 240),            // Snow Storm 2
            explorer_file_selected: Color::Rgb(236, 239, 244),   // Snow Storm 3 for selection
            status_bar: Color::Rgb(163, 190, 140),               // Green
            key: Color::Rgb(136, 192, 208),              // Frost cyan
            string: Color::Rgb(163, 190, 140),           // Green
            number: Color::Rgb(180, 142, 173),           // Purple
            boolean: Color::Rgb(129, 161, 193),          // Blue
            bracket: Color::Rgb(208, 135, 112),          // Orange
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

    #[allow(dead_code)]
    pub fn all_schemes() -> Vec<Self> {
        vec![
            Self::default(),
            Self::monokai(),
            Self::solarized_dark(),
            Self::nord(),
        ]
    }
}
