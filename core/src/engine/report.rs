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

/// What an automation produces for its caller. Deliberately an enum so new
/// output kinds (JSON, multiple files, ...) are one added variant, not a
/// rewrite — the compiler then walks you to every match that must handle them.
///
/// Output is INDEPENDENT of the trigger: a Server automation returns it in the
/// HTTP response, but a Manual one might have an action that emails it, or it's
/// simply discarded. The engine just makes it available.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum OutputValue {
    /// The automation produced nothing (the default).
    #[default]
    None,
    /// A plain string result (a status, an extracted value).
    Text(String),
    /// A file returned as Base64, self-describing via its MIME content type
    /// (e.g. "application/pdf", "image/png", the xlsx MIME). content_type lets
    /// an HTTP response set the right header and the caller know what it got.
    File { content_type: String, base64: String },
}

/// The full record of a run: every action's result, the overall outcome, and
/// whatever output the automation produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub app_name: String,
    pub results: Vec<ActionResult>,
    pub outcome: RunOutcome,
    pub output: OutputValue,
}

impl RunReport {
    /// Did the whole run finish successfully?
    pub fn succeeded(&self) -> bool {
        matches!(self.outcome, RunOutcome::Completed)
    }
}
