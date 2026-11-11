//! The undo history of the editor: a chain of whole-file snapshots (ADR 0043).
//!
//! Each successful operation appends the project file's text after it to the chain; the text before it is
//! the previous link. Undo writes the previous text, redo the next one, and each does so only while the
//! file still holds the text the step started from. If anything else wrote the file in between, nothing is
//! written and the history is dropped, so undo never throws away another program's change.
//!
//! The chain holds one string per step plus one, and the sum of their sizes stays under [`MAX_BYTES`]: the
//! oldest steps go first, but the newest step always stays.

use std::path::Path;

use freemapper_courseplan_model::write_atomic;

/// The most steps kept; the oldest is dropped first.
pub(crate) const MAX_STEPS: usize = 100;
/// The most bytes of snapshots kept in all.
pub(crate) const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HistoryError {
    /// There is no step to undo or redo.
    Nothing,
    /// The file is not what the step expects. Nothing was written and the history is cleared.
    Changed,
    /// Writing the snapshot failed; the file is as it was.
    Write(String),
}

pub(crate) struct UndoHistory {
    /// The texts of the file, oldest first. `states[i + 1]` is the text after the step `labels[i]`.
    states: Vec<String>,
    labels: Vec<String>,
    /// The index in `states` of the text the file holds now (when nothing else wrote it).
    cursor: usize,
    bytes: usize,
    budget: usize,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::with_budget(MAX_BYTES)
    }
}

impl UndoHistory {
    pub(crate) fn with_budget(budget: usize) -> Self {
        UndoHistory {
            states: Vec::new(),
            labels: Vec::new(),
            cursor: 0,
            bytes: 0,
            budget,
        }
    }

    /// The number of steps in the chain, undone ones included.
    #[cfg(test)]
    pub(crate) fn steps(&self) -> usize {
        self.labels.len()
    }

    /// The bytes of snapshots held.
    #[cfg(test)]
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// Records a finished operation. A new step ends the redo line.
    pub(crate) fn record(&mut self, label: String, before: String, after: String) {
        self.states.truncate(self.cursor + 1);
        self.labels.truncate(self.cursor);
        self.bytes = self.states.iter().map(String::len).sum();
        // The chain continues only from the text this step started from.
        if self.states.last() != Some(&before) {
            self.clear();
            self.bytes = before.len();
            self.states.push(before);
        }
        self.bytes += after.len();
        self.states.push(after);
        self.labels.push(label);
        self.cursor = self.states.len() - 1;
        while self.labels.len() > MAX_STEPS || (self.bytes > self.budget && self.labels.len() > 1) {
            self.bytes -= self.states.remove(0).len();
            self.labels.remove(0);
            self.cursor -= 1;
        }
    }

    /// Writes the text before the last step into `path` and returns the step's label.
    pub(crate) fn undo(&mut self, path: &Path) -> Result<String, HistoryError> {
        if self.cursor == 0 {
            return Err(HistoryError::Nothing);
        }
        self.apply(path, self.cursor, self.cursor - 1)?;
        self.cursor -= 1;
        Ok(self.labels[self.cursor].clone())
    }

    /// Writes the text after the last undone step into `path` and returns the step's label.
    pub(crate) fn redo(&mut self, path: &Path) -> Result<String, HistoryError> {
        if self.cursor + 1 >= self.states.len() {
            return Err(HistoryError::Nothing);
        }
        self.apply(path, self.cursor, self.cursor + 1)?;
        self.cursor += 1;
        Ok(self.labels[self.cursor - 1].clone())
    }

    pub(crate) fn clear(&mut self) {
        self.states.clear();
        self.labels.clear();
        self.cursor = 0;
        self.bytes = 0;
    }

    /// The labels of the next undo and the next redo.
    pub(crate) fn labels(&self) -> (Option<&str>, Option<&str>) {
        (
            self.cursor
                .checked_sub(1)
                .and_then(|i| self.labels.get(i))
                .map(String::as_str),
            self.labels.get(self.cursor).map(String::as_str),
        )
    }

    /// Replaces the text `from` by the text `to` in the file, or clears the history when the file holds
    /// something else.
    fn apply(&mut self, path: &Path, from: usize, to: usize) -> Result<(), HistoryError> {
        match std::fs::read(path) {
            Ok(bytes) if bytes == self.states[from].as_bytes() => {}
            _ => {
                self.clear();
                return Err(HistoryError::Changed);
            }
        }
        // A failed write leaves the file as it was, so the step stays available.
        write_atomic(path, &self.states[to]).map_err(|e| HistoryError::Write(e.to_string()))
    }
}
