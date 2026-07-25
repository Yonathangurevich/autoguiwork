use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::models::engine_error::EngineErrorKind;

/// A piece of data that one action produces and another consumes.
/// For now the only thing actions share is a screen position (found images,
/// clicked spots). As the engine grows you'll add more variants here
/// (Text, Number, FilePath...) and the whole system understands them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Pos(u32, u32),
}

/// Where an action gets a position from: either baked into the automation
/// (a literal the user typed) or looked up at run time from a previous action.
///
/// This is the key idea: `MoveTo` no longer demands a hardcoded (x, y).
/// It can say "wherever `found` ended up" and the Context resolves it live.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PosArg {
    Literal(u32, u32),
    Var(String),
}

impl PosArg {
    /// Turn this argument into a concrete (x, y), reading from the context
    /// if it's a variable reference. Errors are the ones you already defined:
    /// a name that was never set, or a variable holding the wrong kind of value.
    pub fn resolve(&self, ctx: &Context) -> Result<(u32, u32), EngineErrorKind> {
        match self {
            PosArg::Literal(x, y) => Ok((*x, *y)),
            PosArg::Var(name) => ctx.get_pos(name),
        }
    }
}

/// The shared scratchpad for a single automation run. The runner creates one,
/// hands `&mut Context` to every action, and drops it when the run ends —
/// so variables never leak between runs.
#[derive(Debug, Default)]
pub struct Context {
    vars: HashMap<String, Value>,

    // Shared cancel flag. The runner checks it between actions; a background
    // watcher thread flips it to true when the user hits the panic key (Esc).
    // Arc = both threads hold a handle to the SAME bool; AtomicBool = flipping
    // it from another thread is safe with no locks. Defaults to false.
    cancel: Arc<AtomicBool>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    /// Hand out a clone of the cancel flag so a watcher thread can set it.
    /// (Cloning an Arc doesn't copy the bool — both handles share one value.)
    pub fn cancel_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
    }

    /// Request cancellation directly (used by tests and, later, the Tauri
    /// "Stop" button — anything that wants to cancel without a keypress).
    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    /// True once cancellation has been requested. The runner calls this
    /// between actions and stops the run if it returns true.
    pub fn is_cancelled(&self) -> bool {
        // Relaxed ordering is fine: we only need to eventually observe the
        // flip, and we're just reading a single bool with no other memory
        // depending on it.
        self.cancel.load(Ordering::Relaxed)
    }

    /// An action stores its result so later actions can use it.
    /// e.g. FindImageLoop does `ctx.set("found", Value::Pos(x, y))`.
    pub fn set(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    /// Read a stored value back as a position. Fails loudly if the name
    /// was never set, or if it holds something that isn't a position.
    pub fn get_pos(&self, name: &str) -> Result<(u32, u32), EngineErrorKind> {
        match self.vars.get(name) {
            None => Err(EngineErrorKind::UnknownVariable {
                name: name.to_string(),
            }),
            Some(Value::Pos(x, y)) => Ok((*x, *y)),
        }
    }
}
