use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = theme_set.themes["base16-ocean.dark"].clone();

        Self {
            syntax_set,
            theme,
        }
    }

    /// Parse content and split into regular text and code blocks
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
            } else {
                // Regular text line
                blocks.push(ContentBlock::Text { text: line });
                i += 1;
            }
        }

        blocks
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
            }
        }

        result_lines
    }

    /// Highlight markdown formatting in text (bold, headers, lists)
    fn highlight_markdown_text(&self, text: &str, default_style: Style) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        // Check for headers (####, ###, ##, etc.)
        if text.trim_start().starts_with('#') {
            let header_end = text.trim_start().chars().take_while(|c| *c == '#').count();
            if header_end > 0 && text.trim_start().chars().nth(header_end) == Some(' ') {
                // Color headers differently based on level
                let color = match header_end {
                    1 => Color::Rgb(255, 100, 100), // Bright red for #
                    2 => Color::Rgb(255, 150, 100), // Orange for ##
                    3 => Color::Rgb(255, 200, 100), // Yellow for ###
                    _ => Color::Rgb(255, 255, 100), // Lighter yellow for ####+
                };
                spans.push(Span::styled(
                    text.to_string(),
                    Style::default().fg(color),
                ));
                return spans;
            }
        }

        // Check for list items (-, *, +)
        let trimmed = text.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let indent_len = text.len() - trimmed.len();
            let indent = &text[..indent_len];
            let marker_and_space = &trimmed[..2];
            let content = &trimmed[2..];

            // Render indent as normal, marker in color, content in default
            if !indent.is_empty() {
                spans.push(Span::styled(indent.to_string(), default_style));
            }
            spans.push(Span::styled(
                marker_and_space.to_string(),
                Style::default().fg(Color::Rgb(100, 200, 255)), // Light blue for list markers
            ));

            // Process the rest for bold
            let content_spans = self.highlight_bold_in_text(content, default_style);
            spans.extend(content_spans);
            return spans;
        }

        // Otherwise, check for bold text
        let highlighted = self.highlight_bold_in_text(text, default_style);
        highlighted
    }

    /// Highlight bold text (**text**)
    fn highlight_bold_in_text(&self, text: &str, default_style: Style) -> Vec<Span<'static>> {
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
                    // Found closing **
                    let bold_text: String = chars[start..end].iter().collect();
                    let full_bold = format!("**{}**", bold_text);

                    spans.push(Span::styled(
                        full_bold,
                        Style::default().fg(Color::Rgb(255, 255, 150)), // Light yellow for bold
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
}
