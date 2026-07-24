use rustautogui::RustAutoGui;
use crate::models::apps::{MoveFiles, OpenApps};
use crate::models::engine_error::{EngineError, EngineErrorKind};
use crate::tools::clickes::{KeyboardOptions, MouseOptions, PcParts};
use crate::tools::find_image::{find_image_loop, sleep_for_f64};
use crate::tools::find_window_title::is_window_open;

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
    MoveLastDownload { to: String },
    WaitForWindow { title: String, waited_ms: f64 },
    FindImageLoop { image_path: String, waited_ms: f64 },
    Sleep(f64),
}

impl ActionsKind {
    pub fn run(&self, gui: &mut RustAutoGui) -> Result<(), EngineErrorKind> {
        match self {
            ActionsKind::Mouse(m) => m.do_it(gui)?,
            ActionsKind::Keyboard(k) => k.do_it(gui)?,
            ActionsKind::Open(app) => app.open()?,
            ActionsKind::MoveLastDownload { to } => {
                MoveFiles::move_last_download_to(to.clone())?
            }
            ActionsKind::WaitForWindow { title , waited_ms} => {
                let mut max = *waited_ms;
                while !is_window_open(title) {
                    if max <= 0.0 {
                        EngineErrorKind::WindowNotFound {
                            title: title.clone(),
                            waited_ms: max,
                        };
                        break;
                    }
                    sleep_for_f64(0.2);
                    max -= 0.2;
                }
            }
            ActionsKind::FindImageLoop { image_path, waited_ms } => {
                find_image_loop(image_path, gui).map_err(|_| EngineErrorKind::ImageNotFound {
                    path: image_path.clone(),
                    waited_ms: *waited_ms,
                })?;
            }
            ActionsKind::Sleep(sec) => sleep_for_f64(*sec),
        };
        Ok(())
    }
}

impl std::fmt::Display for ActionsKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ActionsKind::Mouse(_) => format!("Mouse"),
            ActionsKind::Keyboard(_) => format!("Keyboard"),
            ActionsKind::Open(o) => format!("Open - {}", o.to_string()),
            ActionsKind::MoveLastDownload { to } => format!("MoveLastDownload to {to}"),
            ActionsKind::WaitForWindow { title , waited_ms} => format!("WaitForWindow: {title} for {waited_ms}"),
            ActionsKind::FindImageLoop { image_path , waited_ms} => format!("FindImageLoop: {image_path} for {waited_ms}"),
            ActionsKind::Sleep(_) => format!("Sleep"),
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
        let actions = &self.actions;
        for action in actions {
            action.action.run(gui).map_err(|e| EngineError {
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
