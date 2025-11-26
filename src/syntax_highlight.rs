use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use crate::config::colorscheme::ColorScheme;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
    colorscheme: ColorScheme,
}

impl SyntaxHighlighter {
    pub fn new(colorscheme: ColorScheme) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self {
            syntax_set,
            theme,
            colorscheme,
        }
    }

    /// Update the colorscheme (used when user changes colorscheme)
    pub fn update_colorscheme(&mut self, colorscheme: ColorScheme) {
        self.colorscheme = colorscheme;
    }

    /// Parse content and split into regular text, code blocks, and tables
    pub fn parse_content<'a>(&self, content: &'a str) -> Vec<ContentBlock<'a>> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Check if this is the start of a code block
            if line.trim_start().starts_with("```") {
                let lang_str = line.trim_start()[3..].trim();
                let lang = if lang_str.is_empty() {
                    None
                } else {
                    Some(lang_str)
                };

                // Collect code block lines
                i += 1;
                let start = i;
                while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                    i += 1;
                }

                let code_lines: Vec<&str> = lines[start..i].to_vec();
                let code = code_lines.join("\n");

                blocks.push(ContentBlock::Code { code, lang });

                // Skip closing ```
                i += 1;
            } else if self.is_table_line(line) {
                // Check if this is the start of a table
                let start = i;
                while i < lines.len() && self.is_table_line(lines[i]) {
                    i += 1;
                }
                let table_lines: Vec<&str> = lines[start..i].to_vec();
                blocks.push(ContentBlock::Table { lines: table_lines });
            } else {
                // Regular text line
                blocks.push(ContentBlock::Text { text: line });
                i += 1;
            }
        }

        blocks
    }

    /// Check if a line is part of a markdown table
    fn is_table_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains('|') && !trimmed.is_empty()
    }

    /// Highlight a code block
    pub fn highlight_code(&self, code: &str, lang: Option<&str>) -> Vec<Line<'static>> {
        let syntax = lang
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .or_else(|| self.syntax_set.find_syntax_by_first_line(code))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let highlighted = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();

            let spans: Vec<Span> = highlighted
                .into_iter()
                .map(|(style, text)| {
                    let fg = Color::Rgb(
                        style.foreground.r,
                        style.foreground.g,
                        style.foreground.b,
                    );

                    let mut ratatui_style = Style::default().fg(fg);

                    if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                    }
                    if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                    }
                    if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
                        ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
                    }

                    Span::styled(text.to_string(), ratatui_style)
                })
                .collect();

            lines.push(Line::from(spans));
        }

        lines
    }

    /// Highlight a table line (just color the | characters)
    fn highlight_table_line(&self, line: &str, default_style: Style) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut current_text = String::new();
        let pipe_style = Style::default().fg(self.colorscheme.md_header);

        for ch in line.chars() {
            if ch == '|' {
                if !current_text.is_empty() {
                    spans.push(Span::styled(current_text.clone(), default_style));
                    current_text.clear();
                }
                spans.push(Span::styled("|".to_string(), pipe_style));
            } else {
                current_text.push(ch);
            }
        }

        if !current_text.is_empty() {
            spans.push(Span::styled(current_text, default_style));
        }

        spans
    }

    /// Render content with syntax highlighting
    pub fn render_lines(
        &self,
        content: &str,
        default_style: Style,
    ) -> Vec<Line<'static>> {
        let blocks = self.parse_content(content);
        let mut result_lines = Vec::new();

        for block in blocks {
            match block {
                ContentBlock::Text { text } => {
                    // Apply markdown highlighting to text
                    let highlighted_spans = self.highlight_markdown_text(text, default_style);
                    result_lines.push(Line::from(highlighted_spans));
                }
                ContentBlock::Code { code, lang } => {
                    let highlighted = self.highlight_code(&code, lang);
                    result_lines.extend(highlighted);
                }
                ContentBlock::Table { lines } => {
                    for line in lines {
                        let highlighted = self.highlight_table_line(line, default_style);
                        result_lines.push(Line::from(highlighted));
                    }
                }
            }
        }

        result_lines
    }

    /// Render markdown formatting in text (bold, headers, lists)
    /// This actually renders the markdown (removes markers) instead of just highlighting
    fn highlight_markdown_text(&self, text: &str, default_style: Style) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        // Check for headers (####, ###, ##, etc.) - render without # markers
        if text.trim_start().starts_with('#') {
            let trimmed = text.trim_start();
            let header_end = trimmed.chars().take_while(|c| *c == '#').count();
            if header_end > 0 && trimmed.chars().nth(header_end) == Some(' ') {
                // Extract header content without # markers
                let header_content = &trimmed[header_end + 1..];
                // Render with header style and BOLD modifier
                spans.push(Span::styled(
                    header_content.to_string(),
                    Style::default().fg(self.colorscheme.md_header).add_modifier(Modifier::BOLD),
                ));
                return spans;
            }
        }

        // Check for list items (-, *, +)
        let trimmed = text.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let indent_len = text.len() - trimmed.len();
            let indent = &text[..indent_len];
            let content = &trimmed[2..];

            // Render indent as normal
            if !indent.is_empty() {
                spans.push(Span::styled(indent.to_string(), default_style));
            }
            // Render bullet point
            spans.push(Span::styled(
                "• ".to_string(),
                Style::default().fg(self.colorscheme.md_url),
            ));

            // Process the rest for bold (render mode)
            let content_spans = self.render_bold_in_text(content, default_style);
            spans.extend(content_spans);
            return spans;
        }

        // Otherwise, check for bold text (render mode)
        let highlighted = self.render_bold_in_text(text, default_style);
        highlighted
    }

    /// Render bold text (**text**) - removes markers and applies bold style
    fn render_bold_in_text(&self, text: &str, default_style: Style) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        let mut current_text = String::new();

        while i < chars.len() {
            // Check for **bold**
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                // Flush current text
                if !current_text.is_empty() {
                    spans.push(Span::styled(current_text.clone(), default_style));
                    current_text.clear();
                }

                // Find closing **
                let start = i + 2;
                let mut end = start;
                while end + 1 < chars.len() {
                    if chars[end] == '*' && chars[end + 1] == '*' {
                        break;
                    }
                    end += 1;
                }

                if end + 1 < chars.len() && chars[end] == '*' && chars[end + 1] == '*' {
                    // Found closing ** - render WITHOUT markers, with bold style
                    let bold_text: String = chars[start..end].iter().collect();

                    spans.push(Span::styled(
                        bold_text,
                        Style::default()
                            .fg(self.colorscheme.md_bold)
                            .add_modifier(Modifier::BOLD),
                    ));
                    i = end + 2;
                } else {
                    // No closing **, treat as normal
                    current_text.push(chars[i]);
                    i += 1;
                }
            } else {
                current_text.push(chars[i]);
                i += 1;
            }
        }

        // Flush remaining text
        if !current_text.is_empty() {
            spans.push(Span::styled(current_text, default_style));
        }

        if spans.is_empty() {
            spans.push(Span::styled(text.to_string(), default_style));
        }

        spans
    }
}

pub enum ContentBlock<'a> {
    Text { text: &'a str },
    Code { code: String, lang: Option<&'a str> },
    Table { lines: Vec<&'a str> },
}
