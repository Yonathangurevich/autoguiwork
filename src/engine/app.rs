use rustautogui::RustAutoGui;
use crate::engine::context::{Context, Value};
use crate::models::apps::{MoveFiles, OpenApps};
use crate::models::engine_error::{EngineError, EngineErrorKind};
use crate::tools::clickes::{KeyboardOptions, MouseOptions, PcParts};
use crate::tools::find_image::{find_image_loop, sleep_for_f64};
use crate::tools::find_window_title::wait_for_new_window;

pub struct Actions {
    pub action_index: u32,
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
    pub fn run(&self, gui: &mut RustAutoGui, ctx: &mut Context) -> Result<(), EngineErrorKind> {
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
                let (x, y) = find_image_loop(image_path, gui).map_err(|_| {
                    EngineErrorKind::ImageNotFound {
                        path: image_path.clone(),
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

pub struct App {
    pub name: String,
    pub actions: Vec<Actions>,
}

impl App {
    pub fn new(name: String) -> Self {
        Self {
            name,
            actions: vec![],
        }
    }

    pub fn add_action(&mut self, action_kind: ActionsKind) {
        let action_index = self.actions.len() as u32;
        let action = Actions::new(action_index, action_kind);
        self.actions.push(action);
    }

    pub fn execute(&self, gui: &mut RustAutoGui) -> Result<(), EngineError> {
        // One fresh Context per run — holds the positions/values that actions
        // discover and share. Dropped when the run ends, so nothing leaks
        // between separate executions of the automation.
        let mut ctx = Context::new();

        let actions = &self.actions;
        for action in actions {
            action.action.run(gui, &mut ctx).map_err(|e| EngineError {
                action_index: action.action_index,
                action_name: action.action_name.clone(),
                kind: e,
            })?;
        }

        Ok(())
    }
}

// let mut new_app = App::new("foo".to_string());
// new_app.add_action(Action::new(ActionsKind::Mouse(MoveTo(0,0))));
// new_app.execute();

#[cfg(test)]
mod tests {
    use crate::engine::app::{ActionsKind::Mouse, App};
    use crate::tools::clickes::MouseOptions;

    #[test]
    fn tesing_creatin_app() {
        let mut new_app = App::new("foo".to_string());
        new_app.add_action(Mouse(MouseOptions::DoubleClick));
    }
}
