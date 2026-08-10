use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rustautogui::RustAutoGui;
use serde::{Deserialize, Serialize};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::engine::context::{Context, TextArg, Value};
use crate::engine::report::{ActionResult, ActionStatus, OutputValue, RunOutcome, RunReport};
use crate::engine::trigger::Trigger;
use crate::models::apps::{MoveFiles, OpenApps};
use crate::models::engine_error::EngineErrorKind;
use crate::tools::clickes::{KeyboardOptions, MouseOptions, PcParts};
use crate::tools::find_image::{find_image_loop, sleep_for_f64};
use crate::tools::find_window_title::wait_for_new_window;

#[derive(Debug, Serialize, Deserialize)]
pub struct Actions {
    // index + name are derived, never trusted from the file: skip them on save,
    // and re-stamp them from the real Vec position on load (see App::reindex).
    // This keeps the index available everywhere at runtime while making it
    // impossible for a hand-edited JSON to disagree with the actual order.
    #[serde(skip)]
    pub action_index: u32,
    #[serde(skip)]
    pub action_name: String,
    pub action: ActionsKind,
}

impl Actions {
    pub fn new(action_index: u32, action: ActionsKind) -> Self {
        Self {
            action_index,
            action_name: action.to_string(),
            action,
        }
    }
}

// Default repeat count for keyboard steps (serde needs a fn for default value).
fn one() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionsKind {
    Mouse(MouseOptions),
    // Keyboard can repeat X times (e.g. press LeftArrow 6 times) via one step.
    // repeat defaults to 1 so older JSON (Keyboard as a plain value) still loads.
    Keyboard {
        opts: KeyboardOptions,
        #[serde(default = "one")]
        repeat: u32,
    },
    Open(OpenApps),
    // `to` and `title` are TextArg so a destination path / window title can come
    // from the caller's args.
    MoveLastDownload { to: TextArg, waited_ms: f64 },
    WaitForWindow { title: TextArg, waited_ms: f64 },
    // store_as: the variable name where the found position is saved, so a later
    // Mouse(MoveTo(Var(store_as))) can click exactly where this image was found.
    FindImageLoop { image_path: String, waited_ms: f64, store_as: String },
    Sleep(f64),
    // Output actions: set what the automation hands back to its caller.
    // SetOutput takes a TextArg (literal or Var) for a plain-text result.
    SetOutput(TextArg),
    // ReadFileAsBase64 reads a file (e.g. the downloaded PDF) and returns it as
    // a Base64 File output tagged with its content type. path is a TextArg so it
    // can be a Var set by an earlier action.
    ReadFileAsBase64 { path: TextArg, content_type: String },
}

impl ActionsKind {
    // base_dir: the automation's folder, so relative image paths resolve to a
    // real absolute path regardless of where the program was launched from.
    pub fn run(
        &self,
        gui: &mut RustAutoGui,
        ctx: &mut Context,
        base_dir: &Path,
    ) -> Result<(), EngineErrorKind> {
        match self {
            ActionsKind::Mouse(m) => m.do_it(gui, ctx)?,
            ActionsKind::Keyboard { opts, repeat } => opts.do_it_for(gui, ctx, *repeat)?,
            ActionsKind::Open(app) => app.open(ctx)?,
            ActionsKind::MoveLastDownload { to, waited_ms } => {
                let to = to.resolve(ctx)?;
                MoveFiles::move_last_download_to(to, *waited_ms)?
            }
            ActionsKind::WaitForWindow { title, waited_ms } => {
                // The waiter now owns the loop, the timeout, AND returning the
                // error via `?` — so a timeout actually fails the action instead
                // of silently reporting success like the old version did.
                let title = title.resolve(ctx)?;
                wait_for_new_window(&title, *waited_ms)?;
            }
            ActionsKind::FindImageLoop { image_path, waited_ms, store_as } => {
                // Join the stored (relative) path to the automation's folder so
                // find_image_loop gets a real absolute path it can open.
                let absolute = base_dir.join(image_path);
                let (x, y) = find_image_loop(&absolute.to_string_lossy(), gui).map_err(|_| {
                    EngineErrorKind::ImageNotFound {
                        path: absolute.to_string_lossy().to_string(),
                        waited_ms: *waited_ms,
                    }
                })?;
                // Hand the found position to later actions through the context.
                ctx.set(store_as, Value::Pos(x, y));
            }
            ActionsKind::Sleep(sec) => sleep_for_f64(*sec),
            ActionsKind::SetOutput(text) => {
                let resolved = text.resolve(ctx)?;
                ctx.set_output(OutputValue::Text(resolved));
            }
            ActionsKind::ReadFileAsBase64 { path, content_type } => {
                let file_path = path.resolve(ctx)?;
                let bytes = std::fs::read(&file_path).map_err(|source| {
                    EngineErrorKind::OutputFileRead {
                        path: file_path.clone(),
                        source,
                    }
                })?;
                let encoded = BASE64.encode(&bytes);
                ctx.set_output(OutputValue::File {
                    content_type: content_type.clone(),
                    base64: encoded,
                });
            }
        };
        Ok(())
    }
}

impl std::fmt::Display for ActionsKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ActionsKind::Mouse(_) => "Mouse".to_string(),
            ActionsKind::Keyboard { repeat, .. } if *repeat > 1 => {
                format!("Keyboard (x{repeat})")
            }
            ActionsKind::Keyboard { .. } => "Keyboard".to_string(),
            ActionsKind::Open(o) => format!("Open - {}", o),
            ActionsKind::MoveLastDownload { to, waited_ms } => format!("MoveLastDownload to {to} (timeout {waited_ms}ms)"),
            ActionsKind::WaitForWindow { title , waited_ms} => format!("WaitForWindow: {title} for {waited_ms}"),
            ActionsKind::FindImageLoop { image_path, waited_ms, store_as } => format!("FindImageLoop: {image_path} for {waited_ms} -> {store_as}"),
            ActionsKind::Sleep(_) => "Sleep".to_string(),
            ActionsKind::SetOutput(_) => "SetOutput".to_string(),
            ActionsKind::ReadFileAsBase64 { content_type, .. } => format!("ReadFileAsBase64 ({content_type})"),
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub name: String,

    // How this automation is invoked (Manual click / Server endpoint / ...).
    // #[serde(default)] means older JSON files without a trigger field still
    // load — they default to Manual.
    #[serde(default)]
    pub trigger: Trigger,

    pub actions: Vec<Actions>,

    // The folder this automation lives in (where its automation.json sits).
    // NOT stored in the file — it's determined by WHERE we loaded from, so a
    // moved/copied automation folder just works. Relative image paths like
    // "assets/step_0.png" are resolved against this at run time.
    #[serde(skip)]
    pub base_dir: PathBuf,
}

impl App {
    pub fn new(name: String) -> Self {
        Self {
            name,
            trigger: Trigger::default(),
            actions: vec![],
            base_dir: PathBuf::new(),
        }
    }

    /// Turn a stored (relative) image path into a real absolute path by joining
    /// it to this automation's folder. If the stored path is already absolute,
    /// join() keeps it as-is — so both styles work.
    pub fn resolve_path(&self, relative: &str) -> PathBuf {
        self.base_dir.join(relative)
    }

    pub fn add_action(&mut self, action_kind: ActionsKind) {
        let action_index = self.actions.len() as u32;
        let action = Actions::new(action_index, action_kind);
        self.actions.push(action);
    }

    /// Re-stamp every action's derived fields (index + name) from its real
    /// position in the Vec. Called after loading, since those fields are
    /// #[serde(skip)] and come back as defaults (0 / empty string).
    fn reindex(&mut self) {
        for (i, action) in self.actions.iter_mut().enumerate() {
            action.action_index = i as u32;
            action.action_name = action.action.to_string();
        }
    }

    /// Serialize this automation to a pretty JSON file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), EngineErrorKind> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self).map_err(|source| {
            EngineErrorKind::AppFileParse {
                path: path.to_string_lossy().to_string(),
                source,
            }
        })?;
        fs::write(path, json).map_err(|source| EngineErrorKind::AppFileIo {
            path: path.to_string_lossy().to_string(),
            source,
        })
    }

    /// Load an automation from a JSON file, then re-stamp the derived fields
    /// so indexes/names are correct regardless of what the file contained.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, EngineErrorKind> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| EngineErrorKind::AppFileIo {
            path: path.to_string_lossy().to_string(),
            source,
        })?;
        let mut app: App = serde_json::from_str(&text).map_err(|source| {
            EngineErrorKind::AppFileParse {
                path: path.to_string_lossy().to_string(),
                source,
            }
        })?;
        app.reindex();
        // Anchor image paths to the folder the JSON lives in, not the cwd.
        app.base_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        Ok(app)
    }

    /// Validate the incoming args against this automation's declared inputs and
    /// seed them into the Context, so actions can read them via Var(name) at any
    /// point in the run. This is the bridge between an outside request (an HTTP
    /// body, shaped as a name->value map) and the Context the engine reads.
    ///
    /// Rules:
    ///   - a declared REQUIRED input missing from `args` → MissingRequiredInput
    ///     (rejected BEFORE the run starts, so the automation never half-runs)
    ///   - declared inputs present in `args` are seeded as Value::Text
    ///   - extra args not declared are ignored (harmless — the automation just
    ///     doesn't use them)
    ///
    /// Manual automations declare no inputs, so this is effectively a no-op for
    /// them (any args passed are simply ignored).
    pub fn seed_inputs(
        &self,
        ctx: &mut Context,
        args: &HashMap<String, String>,
    ) -> Result<(), EngineErrorKind> {
        for spec in self.trigger.declared_inputs() {
            match args.get(&spec.name) {
                Some(value) => ctx.set(&spec.name, Value::Text(value.clone())),
                None if spec.required => {
                    return Err(EngineErrorKind::MissingRequiredInput {
                        name: spec.name.clone(),
                    });
                }
                None => { /* optional input not provided — fine, skip it */ }
            }
        }
        Ok(())
    }

    /// Run every action in order, sharing one Context, and return a full
    /// RunReport of what happened — one ActionResult per action, plus the
    /// overall outcome. The report is returned whether the run succeeds or
    /// fails: a failed run is a normal thing to display, not an error to catch.
    ///
    /// The caller passes the Context in (rather than us making it) so it can
    /// grab the cancel handle and start the Esc watcher before the run begins.
    /// The cancel flag is checked before each action.
    pub fn execute(&self, gui: &mut RustAutoGui, ctx: &mut Context) -> RunReport {
        let mut results: Vec<ActionResult> = Vec::with_capacity(self.actions.len());
        let mut outcome = RunOutcome::Completed;

        let mut iter = self.actions.iter();
        // Run actions until one fails or we're cancelled...
        for action in iter.by_ref() {
            if ctx.is_cancelled() {
                outcome = RunOutcome::Cancelled {
                    at_index: action.action_index,
                };
                // this action didn't run — it and everything after are skipped
                results.push(skipped(action));
                break;
            }

            let started = Instant::now();
            match action.action.run(gui, ctx, &self.base_dir) {
                Ok(()) => results.push(ActionResult {
                    index: action.action_index,
                    name: action.action_name.clone(),
                    status: ActionStatus::Ok,
                    duration_ms: started.elapsed().as_millis(),
                }),
                Err(e) => {
                    results.push(ActionResult {
                        index: action.action_index,
                        name: action.action_name.clone(),
                        status: ActionStatus::Failed(e.to_string()),
                        duration_ms: started.elapsed().as_millis(),
                    });
                    outcome = RunOutcome::Failed {
                        at_index: action.action_index,
                    };
                    break;
                }
            }
        }

        // ...then mark every remaining action as Skipped so the report shows
        // the full action list, not a truncated one.
        for action in iter {
            results.push(skipped(action));
        }

        RunReport {
            app_name: self.name.clone(),
            results,
            outcome,
            // Move whatever the automation produced into the report; the worker
            // (or caller) reads this to send back to whoever triggered the run.
            output: ctx.take_output(),
        }
    }
}

/// An ActionResult for an action that never ran.
fn skipped(action: &Actions) -> ActionResult {
    ActionResult {
        index: action.action_index,
        name: action.action_name.clone(),
        status: ActionStatus::Skipped,
        duration_ms: 0,
    }
}

// let mut new_app = App::new("foo".to_string());
// new_app.add_action(Action::new(ActionsKind::Mouse(MoveTo(0,0))));
// new_app.execute();

#[cfg(test)]
mod tests {
    use crate::engine::app::{ActionsKind, ActionsKind::Mouse, App};
    use crate::engine::context::Context;
    use crate::engine::report::{ActionStatus, RunOutcome};
    use crate::engine::trigger::{InputSpec, Trigger};
    use crate::models::engine_error::EngineErrorKind;
    use crate::tools::clickes::MouseOptions;
    use std::collections::HashMap;

    /// Helper: an App with a Server trigger declaring one required "invoice".
    fn invoice_app() -> App {
        let mut app = App::new("invoices".to_string());
        app.trigger = Trigger::Server {
            endpoint_id: "invoices-test".to_string(),
            inputs: vec![InputSpec {
                name: "invoice".to_string(),
                required: true,
            }],
        };
        app
    }

    /// A new App defaults to the Manual trigger.
    #[test]
    fn new_app_is_manual() {
        let app = App::new("x".to_string());
        assert!(matches!(app.trigger, Trigger::Manual));
    }

    /// The args bridge: a provided input lands in Context as text, readable via
    /// Var later. This is the whole point of Step 3.
    #[test]
    fn seed_inputs_puts_arg_into_context() {
        let app = invoice_app();
        let mut ctx = Context::new();

        let mut args = HashMap::new();
        args.insert("invoice".to_string(), "I260014134".to_string());

        app.seed_inputs(&mut ctx, &args).unwrap();
        assert_eq!(ctx.get_text("invoice").unwrap(), "I260014134");
    }

    /// A missing REQUIRED input is rejected before the run, with the right error.
    #[test]
    fn seed_inputs_rejects_missing_required() {
        let app = invoice_app();
        let mut ctx = Context::new();
        let empty = HashMap::new();

        let err = app.seed_inputs(&mut ctx, &empty).unwrap_err();
        assert!(matches!(
            err,
            EngineErrorKind::MissingRequiredInput { name } if name == "invoice"
        ));
    }

    /// Extra args that the automation didn't declare are ignored (harmless).
    #[test]
    fn seed_inputs_ignores_undeclared_args() {
        let app = invoice_app();
        let mut ctx = Context::new();

        let mut args = HashMap::new();
        args.insert("invoice".to_string(), "C123".to_string());
        args.insert("unexpected".to_string(), "junk".to_string());

        app.seed_inputs(&mut ctx, &args).unwrap();
        assert_eq!(ctx.get_text("invoice").unwrap(), "C123");
        // the undeclared one was never seeded
        assert!(ctx.get_text("unexpected").is_err());
    }

    /// A Manual automation declares no inputs, so seeding is a harmless no-op
    /// even if args are passed.
    #[test]
    fn manual_app_seeding_is_noop() {
        let app = App::new("manual".to_string());
        let mut ctx = Context::new();
        let mut args = HashMap::new();
        args.insert("whatever".to_string(), "x".to_string());

        // No declared inputs → Ok, nothing seeded.
        assert!(app.seed_inputs(&mut ctx, &args).is_ok());
        assert!(ctx.get_text("whatever").is_err());
    }

    /// Full input path: an arg seeded from a request resolves through the exact
    /// TextArg::Var that KeyboardOptions::Input uses — so Input(Var("invoice"))
    /// would type the caller's value.
    #[test]
    fn seeded_arg_resolves_through_input_textarg() {
        use crate::engine::context::TextArg;

        let app = invoice_app();
        let mut ctx = Context::new();
        let mut args = HashMap::new();
        args.insert("invoice".to_string(), "I999".to_string());
        app.seed_inputs(&mut ctx, &args).unwrap();

        // This is what Input(Var("invoice")) does internally.
        let arg = TextArg::Var("invoice".to_string());
        assert_eq!(arg.resolve(&ctx).unwrap(), "I999");
    }

    /// Output path (text): SetOutput reading a seeded arg ends up in the report.
    #[test]
    fn set_output_text_lands_in_report() {
        use crate::engine::context::TextArg;
        use crate::engine::report::OutputValue;

        let mut app = invoice_app();
        app.add_action(ActionsKind::SetOutput(TextArg::Var("invoice".to_string())));

        let mut ctx = Context::new();
        let mut args = HashMap::new();
        args.insert("invoice".to_string(), "done: I999".to_string());
        app.seed_inputs(&mut ctx, &args).unwrap();

        let mut gui = rustautogui::RustAutoGui::new(false).expect("gui init");
        let report = app.execute(&mut gui, &mut ctx);

        assert!(report.succeeded());
        match report.output {
            OutputValue::Text(t) => assert_eq!(t, "done: I999"),
            other => panic!("expected Text output, got {other:?}"),
        }
    }

    /// Output path (file): ReadFileAsBase64 reads a real file and returns it as
    /// a base64 File output with the right content type — the invoice PDF case.
    #[test]
    fn read_file_as_base64_lands_in_report() {
        use crate::engine::context::TextArg;
        use crate::engine::report::OutputValue;
        use base64::Engine as _;

        // Write a small file to encode (stands in for the downloaded PDF).
        let tmp = std::env::temp_dir().join("autoguiwork_out_test.bin");
        let bytes = b"%PDF-1.7 fake pdf bytes";
        std::fs::write(&tmp, bytes).unwrap();

        let mut app = App::new("filetest".to_string());
        app.add_action(ActionsKind::ReadFileAsBase64 {
            path: TextArg::Literal(tmp.to_string_lossy().to_string()),
            content_type: "application/pdf".to_string(),
        });

        let mut ctx = Context::new();
        let mut gui = rustautogui::RustAutoGui::new(false).expect("gui init");
        let report = app.execute(&mut gui, &mut ctx);

        let _ = std::fs::remove_file(&tmp);

        assert!(report.succeeded());
        match report.output {
            OutputValue::File { content_type, base64 } => {
                assert_eq!(content_type, "application/pdf");
                let expected = base64::engine::general_purpose::STANDARD.encode(bytes);
                assert_eq!(base64, expected);
            }
            other => panic!("expected File output, got {other:?}"),
        }
    }

    /// A Server trigger with declared inputs survives serialize -> deserialize.
    #[test]
    fn server_trigger_survives_json_roundtrip() {
        let mut app = App::new("invoices".to_string());
        app.trigger = Trigger::Server {
            endpoint_id: "invoices-a3f9".to_string(),
            inputs: vec![InputSpec {
                name: "invoice".to_string(),
                required: true,
            }],
        };

        let json = serde_json::to_string(&app).unwrap();
        let back: App = serde_json::from_str(&json).unwrap();

        match back.trigger {
            Trigger::Server { endpoint_id, inputs } => {
                assert_eq!(endpoint_id, "invoices-a3f9");
                assert_eq!(inputs.len(), 1);
                assert_eq!(inputs[0].name, "invoice");
                assert!(inputs[0].required);
            }
            _ => panic!("expected Server trigger"),
        }
    }

    /// Backward compat: an old JSON with NO trigger field loads as Manual
    /// (thanks to #[serde(default)]), instead of failing to parse.
    #[test]
    fn old_json_without_trigger_loads_as_manual() {
        let old = r#"{ "name": "legacy", "actions": [] }"#;
        let app: App = serde_json::from_str(old).unwrap();
        assert!(matches!(app.trigger, Trigger::Manual));
    }

    #[test]
    fn tesing_creatin_app() {
        let mut new_app = App::new("foo".to_string());
        new_app.add_action(Mouse(MouseOptions::DoubleClick));
    }

    /// If cancel is requested before the run starts, execute() stops at the very
    /// first action ([0]) — outcome Cancelled, and BOTH actions marked Skipped
    /// (nothing ran, nothing touched the GUI).
    #[test]
    fn cancel_before_start_skips_everything() {
        let mut app = App::new("cancel_test".to_string());
        app.add_action(ActionsKind::Sleep(5.0));
        app.add_action(ActionsKind::Sleep(5.0));

        let mut ctx = Context::new();
        ctx.request_cancel(); // flag set before we run

        let mut gui = rustautogui::RustAutoGui::new(false).expect("gui init");
        let report = app.execute(&mut gui, &mut ctx);

        assert!(matches!(
            report.outcome,
            RunOutcome::Cancelled { at_index: 0 }
        ));
        // Full action list is still reported, all skipped.
        assert_eq!(report.results.len(), 2);
        assert!(report
            .results
            .iter()
            .all(|r| matches!(r.status, ActionStatus::Skipped)));
    }

    /// Prove the cancel flag is SHARED through the handle: setting it via a clone
    /// of cancel_handle() (the way the watcher thread does) is observed by the
    /// runner and produces a Cancelled outcome.
    #[test]
    fn cancel_via_handle_is_observed_by_runner() {
        let mut app = App::new("cancel_test2".to_string());
        app.add_action(ActionsKind::Sleep(5.0));

        let mut ctx = Context::new();
        let handle = ctx.cancel_handle(); // a separate Arc, same underlying bool
        assert!(!ctx.is_cancelled());

        // Flip it through the handle, exactly like the watcher thread would.
        handle.store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(ctx.is_cancelled(), "handle and context must share the flag");

        let mut gui = rustautogui::RustAutoGui::new(false).expect("gui init");
        let report = app.execute(&mut gui, &mut ctx);
        assert!(!report.succeeded());
        assert!(matches!(report.outcome, RunOutcome::Cancelled { .. }));
    }
}
