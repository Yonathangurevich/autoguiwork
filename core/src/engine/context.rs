use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::engine::report::OutputValue;
use crate::models::engine_error::EngineErrorKind;

/// A piece of data that one action produces and another consumes.
/// Positions come from image matching; Text comes from run inputs (the caller's
/// args, e.g. an invoice number) and from actions that produce strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Pos(u32, u32),
    Text(String),
}

impl Value {
    /// The variant name, for type-mismatch error messages.
    fn type_name(&self) -> &'static str {
        match self {
            Value::Pos(..) => "position",
            Value::Text(_) => "text",
        }
    }
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

/// Where an action gets a string from: a literal the user typed into the
/// automation, or a variable looked up at run time (e.g. Var("invoice") to type
/// the arg the caller sent). The text mirror of PosArg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextArg {
    Literal(String),
    Var(String),
}

impl TextArg {
    pub fn resolve(&self, ctx: &Context) -> Result<String, EngineErrorKind> {
        match self {
            TextArg::Literal(s) => Ok(s.clone()),
            TextArg::Var(name) => ctx.get_text(name),
        }
    }
}

impl std::fmt::Display for TextArg {
    // For human-readable step labels: a literal shows its text, a variable
    // shows {name} so it's clear the value comes from an arg at run time.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextArg::Literal(s) => write!(f, "{s}"),
            TextArg::Var(name) => write!(f, "{{{name}}}"),
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

    // What the automation will hand back to its caller. Actions (SetOutput,
    // ReadFileAsBase64) write here; execute() takes it into the RunReport at
    // the end. Defaults to OutputValue::None.
    output: OutputValue,
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

    /// An action sets the automation's output (what goes back to the caller).
    /// The last action to call this wins.
    pub fn set_output(&mut self, output: OutputValue) {
        self.output = output;
    }

    /// Take the output out at the end of the run, leaving None behind.
    /// execute() calls this to move the output into the RunReport.
    pub fn take_output(&mut self) -> OutputValue {
        std::mem::take(&mut self.output)
    }

    /// An action stores its result so later actions can use it.
    /// e.g. FindImageLoop does `ctx.set("found", Value::Pos(x, y))`.
    pub fn set(&mut self, name: &str, value: Value) {
        self.vars.insert(name.to_string(), value);
    }

    /// Read a stored value back as a position. Fails loudly if the name was
    /// never set, or if it holds something that isn't a position (e.g. text).
    pub fn get_pos(&self, name: &str) -> Result<(u32, u32), EngineErrorKind> {
        match self.vars.get(name) {
            None => Err(EngineErrorKind::UnknownVariable {
                name: name.to_string(),
            }),
            Some(Value::Pos(x, y)) => Ok((*x, *y)),
            Some(other) => Err(EngineErrorKind::VariableTypeMismatch {
                name: name.to_string(),
                expected: "position".to_string(),
                actual: other.type_name().to_string(),
            }),
        }
    }

    /// Read a stored value back as text — used for input args (e.g. the invoice
    /// number the caller sent) and any string a previous action produced.
    pub fn get_text(&self, name: &str) -> Result<String, EngineErrorKind> {
        match self.vars.get(name) {
            None => Err(EngineErrorKind::UnknownVariable {
                name: name.to_string(),
            }),
            Some(Value::Text(s)) => Ok(s.clone()),
            Some(other) => Err(EngineErrorKind::VariableTypeMismatch {
                name: name.to_string(),
                expected: "text".to_string(),
                actual: other.type_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The input path: seed Context with a text var (like the caller's invoice
    /// arg), then a TextArg::Var resolves to it — this is how Input(Var("invoice"))
    /// types the caller's value.
    #[test]
    fn text_var_resolves_from_context() {
        let mut ctx = Context::new();
        ctx.set("invoice", Value::Text("I260014134".to_string()));

        let arg = TextArg::Var("invoice".to_string());
        assert_eq!(arg.resolve(&ctx).unwrap(), "I260014134");
    }

    /// A literal TextArg ignores the context entirely.
    #[test]
    fn text_literal_resolves_directly() {
        let ctx = Context::new();
        let arg = TextArg::Literal("hello".to_string());
        assert_eq!(arg.resolve(&ctx).unwrap(), "hello");
    }

    /// Asking for the wrong type is a loud, specific error (not a silent wrong
    /// value): a position var read as text → VariableTypeMismatch.
    #[test]
    fn wrong_type_is_a_mismatch_error() {
        let mut ctx = Context::new();
        ctx.set("spot", Value::Pos(10, 20));

        let err = ctx.get_text("spot").unwrap_err();
        assert!(matches!(
            err,
            EngineErrorKind::VariableTypeMismatch { .. }
        ));
    }

    /// An unset variable is UnknownVariable, distinct from a type mismatch.
    #[test]
    fn missing_var_is_unknown() {
        let ctx = Context::new();
        assert!(matches!(
            ctx.get_text("nope").unwrap_err(),
            EngineErrorKind::UnknownVariable { .. }
        ));
    }
}
