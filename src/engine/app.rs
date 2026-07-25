use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rustautogui::RustAutoGui;
use serde::{Deserialize, Serialize};

use crate::engine::context::{Context, Value};
use crate::engine::report::{ActionResult, ActionStatus, RunOutcome, RunReport};
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ActionsKind {
    Mouse(MouseOptions),
    Keyboard(KeyboardOptions),
    Open(OpenApps),
    MoveLastDownload { to: String, waited_ms: f64 },
    WaitForWindow { title: String, waited_ms: f64 },
    // store_as: the variable name where the found position is saved, so a later
    // Mouse(MoveTo(Var(store_as))) can click exactly where this image was found.
    FindImageLoop { image_path: String, waited_ms: f64, store_as: String },
    Sleep(f64),
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
            ActionsKind::Keyboard(k) => k.do_it(gui, ctx)?,
            ActionsKind::Open(app) => app.open()?,
            ActionsKind::MoveLastDownload { to, waited_ms } => {
                MoveFiles::move_last_download_to(to.clone(), *waited_ms)?
            }
            ActionsKind::WaitForWindow { title, waited_ms } => {
                // The waiter now owns the loop, the timeout, AND returning the
                // error via `?` — so a timeout actually fails the action instead
                // of silently reporting success like the old version did.
                wait_for_new_window(title, *waited_ms)?;
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
        };
        Ok(())
    }
}

impl std::fmt::Display for ActionsKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ActionsKind::Mouse(_) => "Mouse".to_string(),
            ActionsKind::Keyboard(_) => "Keyboard".to_string(),
            ActionsKind::Open(o) => format!("Open - {}", o),
            ActionsKind::MoveLastDownload { to, waited_ms } => format!("MoveLastDownload to {to} (timeout {waited_ms}ms)"),
            ActionsKind::WaitForWindow { title , waited_ms} => format!("WaitForWindow: {title} for {waited_ms}"),
            ActionsKind::FindImageLoop { image_path, waited_ms, store_as } => format!("FindImageLoop: {image_path} for {waited_ms} -> {store_as}"),
            ActionsKind::Sleep(_) => "Sleep".to_string(),
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub name: String,
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
    use crate::tools::clickes::MouseOptions;

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
