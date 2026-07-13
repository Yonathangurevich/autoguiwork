use rustautogui::RustAutoGui;

use crate::models::apps::{ExplorerTools, MoveFiles, OpenApps};
use crate::tools::clickes::{KeyboardOptions, MouseOptions, PcParts};
use crate::tools::find_image::{find_image_loop, sleep_for_f64};
use crate::tools::find_window_title::is_window_open;

pub enum Actions {
    Mouse(MouseOptions),
    Keyboard(KeyboardOptions),
    Open(OpenApps),
    MoveLastDownload { to: String },
    WaitForWindow { title: String },
    FindImageLoop { image_path: String },
    Sleep(f64),
}

impl Actions {
    pub fn run(&self, gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Actions::Mouse(m) => m.do_it(gui)?,
            Actions::Keyboard(k) => k.do_it(gui)?,
            Actions::Open(app) => app.open()?,
            Actions::MoveLastDownload { to } => {
                if let Some(last) = MoveFiles::find_last_downloaded(to.clone()) {
                    ExplorerTools::MoveFile(last).run()?;
                }
            }
            Actions::WaitForWindow { title } => {
                while !is_window_open(title) {
                    sleep_for_f64(0.2);
                }
            }
            Actions::FindImageLoop { image_path } => {
                find_image_loop(image_path, gui)?;
            }
            Actions::Sleep(sec) => sleep_for_f64(*sec),
        };
        Ok(())
    }
}

pub struct App {
    pub name: String,
    pub actions: Vec<Actions>
}

impl App {
    
    pub fn new(name: String) -> Self {
        Self { name, actions: vec![] }
    }

    pub fn add_action(&mut self, action: Actions) {
        self.actions.push(action);
    }
    
    pub fn execute(&self, gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
        // we have here all the actions of the App.
        // ill have to execute each action at a time until the previous one is done.
        
        let actions = &self.actions;
        for action in actions {
            action.run(gui)?;
        };

        Ok(())
        
    }
}

// let mut new_app = App::new("foo".to_string());
// new_app.add_action(Mouse(MouseOptions::DoubleClick));
// new_app.execute();

#[cfg(test)]
    mod tests {
        use crate::engine::app::{Actions::Mouse, App};
        use crate::tools::clickes::MouseOptions;

        #[test]
        fn tesing_creatin_app() {
            let mut new_app = App::new("foo".to_string());
            new_app.add_action(Mouse(MouseOptions::DoubleClick));
        }
    }
