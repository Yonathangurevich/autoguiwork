use rustautogui::RustAutoGui;
use serde::{Deserialize, Serialize};

use crate::engine::context::{Context, PosArg, TextArg};
use crate::models::engine_error::EngineErrorKind;

pub trait PcParts {
    fn do_it(&self, gui: &mut RustAutoGui, ctx: &Context) -> Result<(), EngineErrorKind>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MouseOptions {
    // MoveTo/Drag now take a PosArg: either a literal (x, y) the user typed,
    // or Var("name") meaning "wherever a previous action stored that position".
    MoveTo(PosArg, f32),
    LeftClick,
    RightClick,
    Drag(PosArg, f32),
    DoubleClick,
}

impl PcParts for MouseOptions {
    fn do_it(&self, gui: &mut RustAutoGui, ctx: &Context) -> Result<(), EngineErrorKind> {
        match self {
            MouseOptions::MoveTo(pos, speed) => {
                // resolve() reads the Context if this is a Var, or just hands
                // back the literal. Either way we end up with concrete pixels.
                let (x, y) = pos.resolve(ctx)?;
                gui.move_mouse_to_pos(x, y, *speed)?;
            }

            MouseOptions::LeftClick => {
                gui.left_click()?;
            }

            MouseOptions::RightClick => {
                gui.right_click()?;
            }

            MouseOptions::Drag(pos, speed) => {
                let (x, y) = pos.resolve(ctx)?;
                gui.drag_mouse(x as i32, y as i32, *speed)?;
            }

            MouseOptions::DoubleClick => {
                gui.double_click()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KeyboardOptions {
    // Input takes a TextArg so it can type either a fixed string OR a variable
    // resolved at run time — e.g. Input(Var("invoice")) types the caller's arg.
    Input(TextArg),
    PressKey(Keys),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Keys {
    Enter,
    BackSpace,
    Space,
    Shift,
    LeftArrow,
}

impl PcParts for KeyboardOptions {
    fn do_it(&self, gui: &mut RustAutoGui, ctx: &Context) -> Result<(), EngineErrorKind> {
        match self {
            KeyboardOptions::Input(text) => {
                // Resolve the TextArg (literal or a Context variable) to a string.
                let resolved = text.resolve(ctx)?;
                gui.keyboard_input(&resolved)?;
            }
            KeyboardOptions::PressKey(key) => gui.keyboard_command(key.to_str())?,
        }

        Ok(())
    }
}

impl KeyboardOptions {
    pub fn do_it_for(
        &self,
        gui: &mut RustAutoGui,
        ctx: &Context,
        times: u32,
    ) -> Result<(), EngineErrorKind> {
        match self {
            KeyboardOptions::Input(text) => {
                let resolved = text.resolve(ctx)?;
                for _ in 0..times {
                    gui.keyboard_input(&resolved)?
                }
            }
            KeyboardOptions::PressKey(key) => {
                for _ in 0..times {
                    gui.keyboard_command(key.to_str())?
                }
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
            Keys::Shift => "shift",
            Keys::LeftArrow => "left",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::context::Context;
    use crate::tools::{clickes::*, find_image::sleep_for};

    // Types real keystrokes into whatever window is focused — manual test only.
    #[test]
    #[ignore = "sends real keystrokes to the focused window"]
    fn test_keyboard_typing() -> Result<(), Box<dyn std::error::Error>> {
        // open any text file for it
        let mut gui = rustautogui::RustAutoGui::new(false)?;
        let ctx = Context::new();
        sleep_for(5);
        KeyboardOptions::Input(TextArg::Literal("hello world".to_string())).do_it(&mut gui, &ctx)?;

        Ok(())
    }
}
