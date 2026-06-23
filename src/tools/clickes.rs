use rustautogui::RustAutoGui;

pub trait PcParts {
    fn do_it(&self, gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>>;
}

pub enum MouseOptions {
    MoveTo((u32, u32)),
    LeftClick,
    RightClick,
    Drag((u32, u32)),
}

impl PcParts for MouseOptions {
    
    fn do_it(&self, gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
        
        match self {
            MouseOptions::MoveTo(pos) => {
                gui.move_mouse_to_pos(pos.0, pos.1, 1.0)?;
            },

            MouseOptions::LeftClick => {
                gui.left_click()?;
            },

            MouseOptions::RightClick => {
                gui.right_click()?;
            },

            MouseOptions::Drag(pos) => {
                gui.drag_mouse(pos.0 as i32, pos.1 as i32, 1.0)?;
            }
        }

        Ok(())
    }
}

pub enum KeyboardOptions {
    Input(String),
    PressKey(Keys)
}

pub enum Keys {
    Enter,
    BackSpace,
    Space,
    Shift
}

impl PcParts for KeyboardOptions {
    fn do_it(&self, gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
        
        match self {
            KeyboardOptions::Input(text) => {
                gui.keyboard_input(text)?;
            },
            KeyboardOptions::PressKey(key) => {
                gui.keyboard_command(key.to_str())?
            }
        }

        Ok(())
    }
}

impl Keys {
    fn to_str(&self) -> &str {
        match self {
            Keys::Enter => "enter",
            Keys::BackSpace => "backspace",
            Keys::Space => "space",
            Keys::Shift => "shift"
        }
    }
}