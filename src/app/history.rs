use super::App;

impl App {
    // Add command to history (max 10 entries)
    pub fn add_to_command_history(&mut self, command: String) {
        if command.is_empty() {
            return;
        }
        // Remove duplicate if it exists
        if let Some(pos) = self.command_history.iter().position(|x| x == &command) {
            self.command_history.remove(pos);
        }
        // Add to end
        self.command_history.push(command);
        // Keep only last 10 entries
        if self.command_history.len() > 10 {
            self.command_history.remove(0);
        }
        // Reset index
        self.command_history_index = None;
    }

    // Add search to history (max 10 entries)
    pub fn add_to_search_history(&mut self, search: String) {
        if search.is_empty() {
            return;
        }
        // Remove duplicate if it exists
        if let Some(pos) = self.search_history.iter().position(|x| x == &search) {
            self.search_history.remove(pos);
        }
        // Add to end
        self.search_history.push(search);
        // Keep only last 10 entries
        if self.search_history.len() > 10 {
            self.search_history.remove(0);
        }
        // Reset index
        self.search_history_index = None;
    }

    // Navigate to previous command in history
    pub fn get_previous_command(&mut self) -> Option<String> {
        if self.command_history.is_empty() {
            return None;
        }

        let index = match self.command_history_index {
            None => self.command_history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
        };

        self.command_history_index = Some(index);
        self.command_history.get(index).cloned()
    }

    // Navigate to next command in history
    pub fn get_next_command(&mut self) -> Option<String> {
        if self.command_history.is_empty() {
            return None;
        }

        match self.command_history_index {
            None => None,
            Some(i) if i + 1 < self.command_history.len() => {
                self.command_history_index = Some(i + 1);
                self.command_history.get(i + 1).cloned()
            }
            Some(_) => {
                self.command_history_index = None;
                Some(String::new())
            }
        }
    }

    // Navigate to previous search in history
    pub fn get_previous_search(&mut self) -> Option<String> {
        if self.search_history.is_empty() {
            return None;
        }

        let index = match self.search_history_index {
            None => self.search_history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
        };

        self.search_history_index = Some(index);
        self.search_history.get(index).cloned()
    }

    // Navigate to next search in history
    pub fn get_next_search(&mut self) -> Option<String> {
        if self.search_history.is_empty() {
            return None;
        }

        match self.search_history_index {
            None => None,
            Some(i) if i + 1 < self.search_history.len() => {
                self.search_history_index = Some(i + 1);
                self.search_history.get(i + 1).cloned()
            }
            Some(_) => {
                self.search_history_index = None;
                Some(String::new())
            }
        }
    }
}
