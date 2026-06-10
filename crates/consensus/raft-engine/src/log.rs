use crate::message::LogEntry;

/// The Replicated Log used by the Raft consensus engine.
/// In production, this must be persisted to the WAL before responding.
/// For the initial implementation, this acts as the fast in-memory buffer.
pub struct ReplicatedLog {
    entries: Vec<LogEntry>,
}

impl ReplicatedLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn last_log_index(&self) -> usize {
        if self.entries.is_empty() {
            0
        } else {
            self.entries.last().unwrap().index
        }
    }

    pub fn last_log_term(&self) -> u64 {
        if self.entries.is_empty() {
            0
        } else {
            self.entries.last().unwrap().term
        }
    }

    pub fn get_term_at(&self, index: usize) -> Option<u64> {
        if index == 0 {
            return Some(0);
        }
        self.entries.get(index - 1).map(|e| e.term)
    }

    pub fn get_entries_from(&self, index: usize) -> Vec<LogEntry> {
        if index == 0 {
            self.entries.clone()
        } else if index <= self.entries.len() {
            self.entries[index - 1..].to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn truncate_from(&mut self, index: usize) {
        if index > 0 && index <= self.entries.len() {
            self.entries.truncate(index - 1);
        }
    }

    pub fn append(&mut self, entries: Vec<LogEntry>) {
        self.entries.extend(entries);
    }
}
