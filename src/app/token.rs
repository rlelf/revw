use super::App;
use tiktoken_rs::cl100k_base;

impl App {
    /// Count tokens in Markdown format
    pub fn count_tokens_markdown(&self) -> Result<usize, String> {
        let content = if self.is_markdown_file() && !self.markdown_input.is_empty() {
            &self.markdown_input
        } else {
            // Convert current content to Markdown
            match self.convert_to_markdown() {
                Ok(md) => return Self::count_tokens(&md),
                Err(e) => return Err(format!("Failed to convert to Markdown: {}", e)),
            }
        };

        Self::count_tokens(content)
    }

    /// Count tokens in JSON format
    pub fn count_tokens_json(&self) -> Result<usize, String> {
        Self::count_tokens(&self.json_input)
    }

    /// Count tokens in Toon format
    pub fn count_tokens_toon(&self) -> Result<usize, String> {
        let content = if self.is_toon_file() && !self.toon_input.is_empty() {
            &self.toon_input
        } else {
            // Convert current content to Toon
            match self.convert_to_toon() {
                Ok(toon) => return Self::count_tokens(&toon),
                Err(e) => return Err(format!("Failed to convert to Toon: {}", e)),
            }
        };

        Self::count_tokens(content)
    }

    /// Count tokens using tiktoken cl100k_base (GPT-4 tokenizer)
    fn count_tokens(text: &str) -> Result<usize, String> {
        let bpe = cl100k_base().map_err(|e| format!("Failed to load tokenizer: {}", e))?;
        let tokens = bpe.encode_with_special_tokens(text);
        Ok(tokens.len())
    }

    /// Display token count for all formats
    pub fn show_token_count(&mut self) {
        let json_count = self.count_tokens_json().ok();
        let markdown_count = self.count_tokens_markdown().ok();
        let toon_count = self.count_tokens_toon().ok();

        let mut parts = Vec::new();

        if let Some(count) = json_count {
            parts.push(format!("JSON: {}", count));
        }
        if let Some(count) = markdown_count {
            parts.push(format!("Markdown: {}", count));
        }
        if let Some(count) = toon_count {
            parts.push(format!("Toon: {}", count));
        }

        if parts.is_empty() {
            self.set_status("Token count error: Failed to count tokens");
        } else {
            self.set_status(&format!("Tokens - {}", parts.join(", ")));
        }
    }

    /// Print token count for all formats to stdout
    pub fn print_token_count(&self) {
        let json_count = self.count_tokens_json().ok();
        let markdown_count = self.count_tokens_markdown().ok();
        let toon_count = self.count_tokens_toon().ok();

        println!("Token counts:");
        if let Some(count) = json_count {
            println!("  JSON:     {}", count);
        }
        if let Some(count) = markdown_count {
            println!("  Markdown: {}", count);
        }
        if let Some(count) = toon_count {
            println!("  Toon:     {}", count);
        }
    }
}
