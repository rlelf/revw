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
                    result_lines.push(Line::styled(text.to_string(), default_style));
                }
                ContentBlock::Code { code, lang } => {
                    let highlighted = self.highlight_code(&code, lang);
                    result_lines.extend(highlighted);
                }
            }
        }

        result_lines
    }
}

pub enum ContentBlock<'a> {
    Text { text: &'a str },
    Code { code: String, lang: Option<&'a str> },
}
