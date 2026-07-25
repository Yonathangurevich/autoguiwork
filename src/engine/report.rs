//! The record of a single automation run — what ran, how it went, how long.
//!
//! execute() returns one of these instead of a bare Result. A UI renders it as
//! a progress panel; the headless worker can log it to debug remote failures.
//! Crucially it's returned whether the run SUCCEEDS or FAILS — a failed run is
//! a normal report to display, not an exception to catch.

use serde::{Deserialize, Serialize};

/// How one action turned out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    /// Ran and succeeded.
    Ok,
    /// Ran and failed, with the error message.
    Failed(String),
    /// Never ran because an earlier action failed or the run was cancelled.
    Skipped,
}

/// The outcome of one action, for the run log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub index: u32,
    pub name: String,
    pub status: ActionStatus,
    /// Wall-clock time this action took, in milliseconds (0 for skipped ones).
    pub duration_ms: u128,
}

/// The one-glance summary of a whole run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunOutcome {
    /// Every action succeeded.
    Completed,
    /// An action failed; `at_index` is where it stopped.
    Failed { at_index: u32 },
    /// The user (or Stop button) cancelled; `at_index` is where it stopped.
    Cancelled { at_index: u32 },
}

/// The full record of a run: every action's result plus the overall outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub app_name: String,
    pub results: Vec<ActionResult>,
    pub outcome: RunOutcome,
}

impl RunReport {
    /// Did the whole run finish successfully?
    pub fn succeeded(&self) -> bool {
        matches!(self.outcome, RunOutcome::Completed)
    }
}
