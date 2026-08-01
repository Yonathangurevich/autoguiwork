use rustautogui::RustAutoGui;

use crate::tools::{clickes::PcParts, find_image::find_image_loop};

pub struct Step {
    pub position: u32,
    pub image: String,
    pub image_location: (u32, u32),
    pub actions: Vec<Box<dyn PcParts>>,
}

impl Default for Step {
    fn default() -> Self {
        Self::new()
    }
}

impl Step {
    pub fn new() -> Self {
        Self {
            position: 0,
            image: String::new(),
            image_location: (0, 0),
            actions: vec![],
        }
    }

    pub fn add_image(&mut self, image: &str, gui: &mut RustAutoGui) {
        self.image = image.to_string();
        if let Ok(image_location) = find_image_loop(image, gui) {
            self.image_location = image_location;
        }
    }

    pub fn add_action<T: PcParts + 'static>(&mut self, actions: T) {
        let new_box = Box::new(actions);
        self.actions.push(new_box);
    }
}

// let new step = Step::new();
// step.add_image("blah.png");
// step.add_action(MouseOptions::MoveTo(step.image_locatian));
// step.add_action(MouseOptions::LeftClick);
// step.execute();
