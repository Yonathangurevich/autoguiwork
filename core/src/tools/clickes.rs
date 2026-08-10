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
    // Common
    Enter,
    BackSpace,
    Space,
    Tab,
    Escape,
    Delete,
    Insert,
    // Arrows
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    // Navigation
    Home,
    End,
    PageUp,
    PageDown,
    // Modifiers
    Shift,
    Ctrl,
    Alt,
    Win,
    // Function keys F1–F24
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
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
    // Maps each key to the exact string rustautogui's keyboard_command accepts.
    fn to_str(&self) -> &str {
        match self {
            Keys::Enter => "enter",
            Keys::BackSpace => "backspace",
            Keys::Space => "space",
            Keys::Tab => "tab",
            Keys::Escape => "escape",
            Keys::Delete => "delete",
            Keys::Insert => "insert",
            Keys::LeftArrow => "left",
            Keys::RightArrow => "right",
            Keys::UpArrow => "up",
            Keys::DownArrow => "down",
            Keys::Home => "home",
            Keys::End => "end",
            Keys::PageUp => "pgup",
            Keys::PageDown => "pgdn",
            Keys::Shift => "shift",
            Keys::Ctrl => "ctrl",
            Keys::Alt => "alt",
            Keys::Win => "win",
            Keys::F1 => "f1",
            Keys::F2 => "f2",
            Keys::F3 => "f3",
            Keys::F4 => "f4",
            Keys::F5 => "f5",
            Keys::F6 => "f6",
            Keys::F7 => "f7",
            Keys::F8 => "f8",
            Keys::F9 => "f9",
            Keys::F10 => "f10",
            Keys::F11 => "f11",
            Keys::F12 => "f12",
            Keys::F13 => "f13",
            Keys::F14 => "f14",
            Keys::F15 => "f15",
            Keys::F16 => "f16",
            Keys::F17 => "f17",
            Keys::F18 => "f18",
            Keys::F19 => "f19",
            Keys::F20 => "f20",
            Keys::F21 => "f21",
            Keys::F22 => "f22",
            Keys::F23 => "f23",
            Keys::F24 => "f24",
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
