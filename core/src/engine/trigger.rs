//! How an automation is started. Every App declares exactly one Trigger — the
//! n8n model: a workflow has one start point that decides how it's invoked and
//! what data flows in.
//!
//! Only Manual and Server exist for now. The enum is deliberately open-ended:
//! future triggers (Schedule { cron }, AppSpecific { source: Outlook, ... },
//! Polling { ... }) are each a single new variant — the compiler will then walk
//! you to every match that needs to handle them, with no wider refactor.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// One input an automation expects when triggered — the contract between the
/// outside world (an HTTP body) and the Context variables the actions read.
/// Used to validate a request BEFORE running and to build the UI form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    /// The variable name actions read via Var(name) — e.g. "invoice".
    pub name: String,
    /// If true, a trigger call missing this input is rejected before the run.
    pub required: bool,
}

/// How an automation gets invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    /// Run by a manual click (or `run <name>`). Takes no input.
    Manual,

    /// Exposed as an API endpoint. A caller POSTs a body; `inputs` declares the
    /// fields the automation expects. An empty `inputs` means "accepts no body"
    /// — still a valid server automation (like an n8n webhook with no params).
    ///
    /// endpoint_id is the stable URL segment (POST /run/<endpoint_id>). It's
    /// generated ONCE when the user creates the worker server and saved in the
    /// automation's JSON, so the URL n8n calls survives restarts.
    Server {
        endpoint_id: String,
        inputs: Vec<InputSpec>,
    },
}

impl Trigger {
    /// The inputs this trigger declares (empty for Manual).
    pub fn declared_inputs(&self) -> &[InputSpec] {
        match self {
            Trigger::Manual => &[],
            Trigger::Server { inputs, .. } => inputs,
        }
    }

    /// The endpoint id, if this automation is exposed as a server.
    pub fn endpoint_id(&self) -> Option<&str> {
        match self {
            Trigger::Manual => None,
            Trigger::Server { endpoint_id, .. } => Some(endpoint_id),
        }
    }
}

/// Build a stable, URL-safe endpoint id from an automation's name:
/// a slug of the name plus a short random suffix for uniqueness.
/// e.g. "Invoice Sync" -> "invoice-sync-a3f9"; a name that slugs to nothing
/// (e.g. all-Hebrew) falls back to "app-a3f9".
pub fn generate_endpoint_id(name: &str) -> String {
    // Slugify: keep ASCII alphanumerics, turn runs of anything else into '-'.
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    let base = if slug.is_empty() { "app" } else { slug };

    // 4-char base36 suffix (0-9a-z) — short but plenty for a personal tool.
    let mut rng = rand::thread_rng();
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let suffix: String = (0..4)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect();

    format!("{base}-{suffix}")
}

impl Default for Trigger {
    /// A brand-new automation is Manual until the user chooses otherwise.
    fn default() -> Self {
        Trigger::Manual
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_a_normal_name() {
        let id = generate_endpoint_id("Invoice Sync!");
        assert!(id.starts_with("invoice-sync-"), "got {id}");
        // slug + '-' + 4-char suffix
        assert_eq!(id.len(), "invoice-sync-".len() + 4);
    }

    #[test]
    fn falls_back_for_nonascii_name() {
        // An all-Hebrew name slugs to nothing → "app-xxxx".
        let id = generate_endpoint_id("צפי הגעת מוצרים");
        assert!(id.starts_with("app-"), "got {id}");
        assert_eq!(id.len(), "app-".len() + 4);
    }

    #[test]
    fn suffix_makes_it_unique() {
        // Two ids from the same name should (almost surely) differ by suffix.
        let a = generate_endpoint_id("same");
        let b = generate_endpoint_id("same");
        assert_ne!(a, b);
    }
}
