use std::error;

use rustautogui::RustAutoGui;

use crate::{
    engine::context::{Context, TextArg},
    models::apps::OpenApps,
    tools::{
        clickes::{KeyboardOptions, Keys, PcParts},
        find_image::sleep_for_f64,
        find_window_title::is_window_open,
    },
};

pub fn invoics_app(
    invoice: String,
    gui: &mut RustAutoGui,
) -> Result<String, Box<dyn error::Error>> {
    if invoice.starts_with("I") {
        let ctx = Context::new();
        OpenApps::RandomApp(TextArg::Literal(
            "priority:priform@AINVOICES::.:tabula.ini:1".to_string(),
        ))
        .open(&ctx)?;

        // knows when the window is opened based on the title name of the app.
        while !is_window_open("חשבוניות מס - גל-אור - jonatan") {
            println!("closed");
            sleep_for_f64(0.2);
        }

        for _ in 0..4 {
            KeyboardOptions::PressKey(Keys::LeftArrow).do_it(gui, &ctx)?;
        }

        Ok("opne".to_string())
    } else if invoice.starts_with("C") {
        Ok("()".to_string())
    } else {
        Ok("none".to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::applictions::invoics_app;

    // Launches Priority and WAITS for its window — would hang without the ERP.
    #[test]
    #[ignore = "launches Priority ERP and waits for its window"]
    fn test_if_windows_is_opening() -> Result<(), Box<dyn std::error::Error>> {
        let mut gui = rustautogui::RustAutoGui::new(false)?;
        invoics_app("Iblah".to_string(), &mut gui)?;

        Ok(())
    }
}
