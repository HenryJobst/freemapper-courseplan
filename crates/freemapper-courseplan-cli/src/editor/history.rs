//! The undo history of the editor: whole-file snapshots (ADR 0043).
//!
//! Each successful operation records the project file's text before and after. Undo writes `before`, redo
//! writes `after`, and each does so only while the file still holds the text the step started from. If
//! anything else wrote the file in between, nothing is written and the history is dropped, so undo never
//! throws away another program's change.

use std::io::Write as _;
use std::path::Path;

/// The most steps kept; the oldest is dropped first.
pub(crate) const MAX_STEPS: usize = 100;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HistoryError {
    /// There is no step to undo or redo.
    Nothing,
    /// The file is not what the step expects. Nothing was written and the history is cleared.
    Changed,
    /// Writing the snapshot failed; the file is as it was.
    Write(String),
}

struct Step {
    label: String,
    before: String,
    after: String,
}

#[derive(Default)]
pub(crate) struct UndoHistory {
    undo: Vec<Step>,
    redo: Vec<Step>,
}

impl UndoHistory {
    /// Records a finished operation. A new step ends the redo line.
    pub(crate) fn record(&mut self, label: String, before: String, after: String) {
        self.redo.clear();
        self.undo.push(Step {
            label,
            before,
            after,
        });
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
    }

    /// Writes the text before the last step into `path` and returns the step's label.
    pub(crate) fn undo(&mut self, path: &Path) -> Result<String, HistoryError> {
        let step = self.undo.pop().ok_or(HistoryError::Nothing)?;
        self.apply(path, &step.after, &step.before)
            .inspect_err(|e| {
                // A failed write leaves the file as it was, so the step stays available.
                if matches!(e, HistoryError::Write(_)) {
                    self.undo.push(Step {
                        label: step.label.clone(),
                        before: step.before.clone(),
                        after: step.after.clone(),
                    });
                }
            })?;
        let label = step.label.clone();
        self.redo.push(step);
        Ok(label)
    }

    /// Writes the text after the last undone step into `path` and returns the step's label.
    pub(crate) fn redo(&mut self, path: &Path) -> Result<String, HistoryError> {
        let step = self.redo.pop().ok_or(HistoryError::Nothing)?;
        self.apply(path, &step.before, &step.after)
            .inspect_err(|e| {
                if matches!(e, HistoryError::Write(_)) {
                    self.redo.push(Step {
                        label: step.label.clone(),
                        before: step.before.clone(),
                        after: step.after.clone(),
                    });
                }
            })?;
        let label = step.label.clone();
        self.undo.push(step);
        Ok(label)
    }

    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// The labels of the next undo and the next redo.
    pub(crate) fn labels(&self) -> (Option<&str>, Option<&str>) {
        (
            self.undo.last().map(|s| s.label.as_str()),
            self.redo.last().map(|s| s.label.as_str()),
        )
    }

    /// Replaces `expected` by `target` in the file, or clears the history when the file holds something else.
    fn apply(&mut self, path: &Path, expected: &str, target: &str) -> Result<(), HistoryError> {
        match std::fs::read(path) {
            Ok(bytes) if bytes == expected.as_bytes() => {}
            _ => {
                self.clear();
                return Err(HistoryError::Changed);
            }
        }
        write_atomic(path, target).map_err(|e| HistoryError::Write(e.to_string()))
    }
}

/// Writes `text` next to `path` and renames it over the file, so a failure leaves the old file.
fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".undo.tmp");
    let tmp = path.with_file_name(name);
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
