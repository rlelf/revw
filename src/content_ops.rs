/// Unified interface for content operations (JSON and Markdown)
pub trait ContentOperations {
    /// Add a new inside entry
    fn add_inside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String>;

    /// Add a new outside entry
    fn add_outside_entry(&self, content: &str) -> Result<(String, usize, usize, String), String>;

    /// Delete an entry at the cursor position
    fn delete_entry_at_cursor(
        &self,
        content: &str,
        cursor_line: usize,
        lines: &[String],
    ) -> Result<(String, String), String>;

    /// Duplicate an entry at the cursor position
    fn duplicate_entry_at_cursor(
        &self,
        content: &str,
        cursor_line: usize,
        lines: &[String],
    ) -> Result<(String, String), String>;

    /// Order entries (outside by percentage desc + name asc, inside by date desc)
    fn order_entries(&self, content: &str) -> Result<(String, String), String>;

    /// Order entries by percentage only (outside) and date (inside)
    fn order_by_percentage(&self, content: &str) -> Result<(String, String), String>;

    /// Order entries by name only (outside) and date (inside)
    fn order_by_name(&self, content: &str) -> Result<(String, String), String>;
}
