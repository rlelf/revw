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
    pub text_dim: Color,                     // Dimmed text color
    pub line_number: Color,                  // Line number color (Edit mode with set number)
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
            background: Color::Black,
            border: Color::DarkGray,
            window_border: Color::DarkGray,
            window_title: Color::Cyan,
            explorer_border: Color::DarkGray,
            explorer_title: Color::Cyan,
            card_border: Color::DarkGray,
            text: Color::White,
            text_dim: Color::DarkGray,
            line_number: Color::Yellow,
            highlight: Color::Yellow,
            selected: Color::Cyan,
            card_selected: Color::Yellow,
            card_visual: Color::Magenta,
            card_title: Color::Cyan,
            card_content: Color::White,
            overlay_field_active: Color::Yellow,
            overlay_field_selected: Color::Cyan,
            overlay_field_placeholder: Color::DarkGray,
            overlay_field_normal: Color::White,
            explorer_folder: Color::Cyan,
            explorer_file: Color::White,
            explorer_file_selected: Color::Yellow,
            status_bar: Color::White,
            key: Color::Cyan,
            string: Color::Magenta,
            number: Color::Magenta,
            boolean: Color::Yellow,
            bracket: Color::Yellow,
        }
    }

    pub fn morning() -> Self {
        Self {
            name: "Morning",
            background: Color::White,
            border: Color::Black,
            window_border: Color::Black,
            window_title: Color::Blue,
            explorer_border: Color::Black,
            explorer_title: Color::DarkGray,
            card_border: Color::DarkGray,
            text: Color::Black,
            text_dim: Color::DarkGray,
            line_number: Color::DarkGray,
            highlight: Color::Yellow,
            selected: Color::Cyan,
            card_selected: Color::Blue,
            card_visual: Color::Magenta,
            card_title: Color::Blue,
            card_content: Color::Black,
            overlay_field_active: Color::Red,
            overlay_field_selected: Color::Blue,
            overlay_field_placeholder: Color::Gray,
            overlay_field_normal: Color::Black,
            explorer_folder: Color::Blue,
            explorer_file: Color::Black,
            explorer_file_selected: Color::Red,
            status_bar: Color::Blue,
            key: Color::Blue,
            string: Color::Red,
            number: Color::Magenta,
            boolean: Color::DarkGray,
            bracket: Color::Black,
        }
    }

    pub fn evening() -> Self {
        Self {
            name: "Evening",
            background: Color::Rgb(50, 50, 70),
            border: Color::Rgb(120, 120, 140),
            window_border: Color::Rgb(120, 120, 140),
            window_title: Color::Rgb(200, 200, 255),
            explorer_border: Color::Rgb(120, 120, 140),
            explorer_title: Color::Rgb(150, 200, 255),
            card_border: Color::Rgb(100, 100, 120),
            text: Color::Rgb(220, 220, 255),
            text_dim: Color::Rgb(140, 140, 160),
            line_number: Color::Rgb(140, 140, 160),
            highlight: Color::Yellow,
            selected: Color::Cyan,
            card_selected: Color::Rgb(255, 200, 100),
            card_visual: Color::Rgb(150, 200, 255),
            card_title: Color::Rgb(150, 200, 255),
            card_content: Color::Rgb(220, 220, 255),
            overlay_field_active: Color::Rgb(255, 200, 100),
            overlay_field_selected: Color::Rgb(150, 200, 255),
            overlay_field_placeholder: Color::Rgb(140, 140, 160),
            overlay_field_normal: Color::Rgb(220, 220, 255),
            explorer_folder: Color::Rgb(150, 200, 255),
            explorer_file: Color::Rgb(200, 200, 220),
            explorer_file_selected: Color::Rgb(255, 255, 255),
            status_bar: Color::Rgb(150, 200, 255),
            key: Color::Rgb(150, 200, 255),
            string: Color::Rgb(255, 150, 150),
            number: Color::Rgb(200, 150, 255),
            boolean: Color::Rgb(200, 200, 150),
            bracket: Color::Rgb(180, 180, 200),
        }
    }

    pub fn pablo() -> Self {
        Self {
            name: "Pablo",
            background: Color::Black,
            border: Color::Rgb(100, 100, 100),
            window_border: Color::Rgb(100, 100, 100),
            window_title: Color::Cyan,
            explorer_border: Color::Rgb(100, 100, 100),
            explorer_title: Color::Yellow,
            card_border: Color::Rgb(80, 80, 80),
            text: Color::White,
            text_dim: Color::DarkGray,
            line_number: Color::DarkGray,
            highlight: Color::Yellow,
            selected: Color::Cyan,
            card_selected: Color::Yellow,
            card_visual: Color::Magenta,
            card_title: Color::Cyan,
            card_content: Color::White,
            overlay_field_active: Color::Yellow,
            overlay_field_selected: Color::Cyan,
            overlay_field_placeholder: Color::DarkGray,
            overlay_field_normal: Color::White,
            explorer_folder: Color::Cyan,
            explorer_file: Color::Gray,
            explorer_file_selected: Color::Yellow,
            status_bar: Color::Green,
            key: Color::Cyan,
            string: Color::Red,
            number: Color::Magenta,
            boolean: Color::Yellow,
            bracket: Color::White,
        }
    }

    pub fn ron() -> Self {
        Self {
            name: "Ron",
            background: Color::Rgb(0, 0, 0),
            border: Color::Rgb(135, 135, 135),
            window_border: Color::Rgb(135, 135, 135),
            window_title: Color::Rgb(175, 215, 255),
            explorer_border: Color::Rgb(135, 135, 135),
            explorer_title: Color::Rgb(255, 215, 0),
            card_border: Color::Rgb(95, 95, 95),
            text: Color::Rgb(215, 215, 215),
            text_dim: Color::Rgb(135, 135, 135),
            line_number: Color::Rgb(135, 135, 135),
            highlight: Color::Rgb(255, 215, 0),
            selected: Color::Rgb(0, 175, 215),
            card_selected: Color::Rgb(255, 215, 0),
            card_visual: Color::Rgb(215, 95, 255),
            card_title: Color::Rgb(175, 215, 255),
            card_content: Color::Rgb(215, 215, 215),
            overlay_field_active: Color::Rgb(255, 215, 0),
            overlay_field_selected: Color::Rgb(0, 175, 215),
            overlay_field_placeholder: Color::Rgb(135, 135, 135),
            overlay_field_normal: Color::Rgb(215, 215, 215),
            explorer_folder: Color::Rgb(175, 215, 255),
            explorer_file: Color::Rgb(175, 175, 175),
            explorer_file_selected: Color::Rgb(255, 255, 255),
            status_bar: Color::Rgb(95, 215, 135),
            key: Color::Rgb(175, 215, 255),
            string: Color::Rgb(255, 135, 135),
            number: Color::Rgb(215, 135, 255),
            boolean: Color::Rgb(255, 215, 0),
            bracket: Color::Rgb(175, 175, 175),
        }
    }

    pub fn blue() -> Self {
        Self {
            name: "Blue",
            background: Color::Rgb(0, 0, 95),
            border: Color::Rgb(95, 135, 175),
            window_border: Color::Rgb(95, 135, 175),
            window_title: Color::Rgb(175, 215, 255),
            explorer_border: Color::Rgb(95, 135, 175),
            explorer_title: Color::Rgb(255, 255, 135),
            card_border: Color::Rgb(60, 95, 135),
            text: Color::Rgb(215, 215, 255),
            text_dim: Color::Rgb(135, 135, 175),
            line_number: Color::Rgb(135, 135, 175),
            highlight: Color::Yellow,
            selected: Color::Cyan,
            card_selected: Color::Rgb(255, 255, 135),
            card_visual: Color::Rgb(255, 95, 215),
            card_title: Color::Rgb(175, 215, 255),
            card_content: Color::Rgb(215, 215, 255),
            overlay_field_active: Color::Rgb(255, 255, 135),
            overlay_field_selected: Color::Rgb(135, 215, 255),
            overlay_field_placeholder: Color::Rgb(135, 135, 175),
            overlay_field_normal: Color::Rgb(215, 215, 255),
            explorer_folder: Color::Rgb(175, 215, 255),
            explorer_file: Color::Rgb(175, 175, 215),
            explorer_file_selected: Color::Rgb(255, 255, 255),
            status_bar: Color::Rgb(135, 215, 255),
            key: Color::Rgb(175, 215, 255),
            string: Color::Rgb(255, 175, 175),
            number: Color::Rgb(215, 175, 255),
            boolean: Color::Rgb(135, 255, 175),
            bracket: Color::Rgb(175, 215, 255),
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "default" => Some(Self::default()),
            "morning" => Some(Self::morning()),
            "evening" => Some(Self::evening()),
            "pablo" => Some(Self::pablo()),
            "ron" => Some(Self::ron()),
            "blue" => Some(Self::blue()),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn all_schemes() -> Vec<Self> {
        vec![
            Self::default(),
            Self::morning(),
            Self::evening(),
            Self::pablo(),
            Self::ron(),
            Self::blue(),
        ]
    }
}
