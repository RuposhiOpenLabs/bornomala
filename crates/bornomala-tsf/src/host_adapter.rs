/// Abstract interface between the Bornomala typing engine and the text host application.
pub trait HostAdapter {
    /// Inserts committed text into the host application.
    fn insert_text(&mut self, text: &str);

    /// Deletes `count` characters immediately before the caret and inserts `text`.
    fn replace_last(&mut self, count: usize, text: &str);

    /// Deletes `count` characters immediately before the caret.
    fn delete_back(&mut self, count: usize);

    /// Sets or updates the active inline composition string.
    fn set_composition(&mut self, text: &str);

    /// Clears any active inline composition string.
    fn clear_composition(&mut self);
}

/// Simulated Host Adapter used in Linux automated integration tests.
#[derive(Debug, Clone, Default)]
pub struct SimulatedHostAdapter {
    pub committed_buffer: String,
    pub composition_buffer: Option<String>,
    pub caret_position: usize,
}

impl SimulatedHostAdapter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HostAdapter for SimulatedHostAdapter {
    fn insert_text(&mut self, text: &str) {
        self.committed_buffer.push_str(text);
        self.caret_position += text.chars().count();
        self.composition_buffer = None;
    }

    fn replace_last(&mut self, count: usize, text: &str) {
        for _ in 0..count {
            self.committed_buffer.pop();
        }
        self.committed_buffer.push_str(text);
        self.caret_position = self.committed_buffer.chars().count();
        self.composition_buffer = None;
    }

    fn delete_back(&mut self, count: usize) {
        for _ in 0..count {
            self.committed_buffer.pop();
        }
        self.caret_position = self.committed_buffer.chars().count();
        self.composition_buffer = None;
    }

    fn set_composition(&mut self, text: &str) {
        self.composition_buffer = Some(text.to_string());
    }

    fn clear_composition(&mut self) {
        self.composition_buffer = None;
    }
}
