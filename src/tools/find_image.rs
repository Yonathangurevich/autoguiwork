use rustautogui::RustAutoGui;
use std::time::Duration;

use crate::tools::clickes::{MouseOptions, PcParts};

pub fn find_image(
    template_path: &str,
    gui: &mut RustAutoGui,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let screen_size = gui.get_screen_size();

    // preparing the template image

    if gui
        .prepare_template_from_file(
            template_path,
            Some((0, 0, screen_size.0 as u32, screen_size.1 as u32)),
            rustautogui::MatchMode::Segmented,
        )
        .is_ok()
    {
        // finding the prepared template image
        let found_location = gui.find_image_on_screen(0.9)?;
        if let Some(location) = found_location {
            let location_x = location[0].0;
            let location_y = location[0].1;

            Ok((location_x, location_y))
        } else {
            Err("image not found on screen".into())
        }
    } else {
        Err("image not found on screen".into())
    }
}

pub fn sleep_for(secs: u64) {
    std::thread::sleep(Duration::from_secs(secs));
}

pub fn sleep_for_f64(secs: f64) {
    std::thread::sleep(Duration::from_secs_f64(secs));
}

pub fn find_image_loop(
    template_path: &str,
    gui: &mut RustAutoGui,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let screen_size = gui.get_screen_size();

    if gui
        .prepare_template_from_file(
            template_path,
            Some((0, 0, screen_size.0 as u32, screen_size.1 as u32)),
            rustautogui::MatchMode::Segmented,
        )
        .is_ok()
    {
        let found_image = gui.loop_find_image_on_screen(0.9, 10)?;
        if let Some(location) = found_image {
            let location_x = location[0].0;
            let location_y = location[0].1;

            Ok((location_x, location_y))
        } else {
            Err("image not found on screen".into())
        }
    } else {
        Err("image not found on screen".into())
    }
}

pub fn find_image_loop_and_move_double_click(
    template_path: &str,
    gui: &mut RustAutoGui,
) -> Result<(), Box<dyn std::error::Error>> {
    let screen_size = gui.get_screen_size();

    if gui
        .prepare_template_from_file(
            template_path,
            Some((0, 0, screen_size.0 as u32, screen_size.1 as u32)),
            rustautogui::MatchMode::Segmented,
        )
        .is_ok()
    {
        let found_image = gui.loop_find_image_on_screen(0.9, 10)?;
        if let Some(location) = found_image {
            let location_x = location[0].0;
            let location_y = location[0].1;

            let full_location = (location_x, location_y);
            MouseOptions::MoveTo(full_location, 0.1).do_it(gui)?;
            MouseOptions::DoubleClick.do_it(gui)?;

            Ok(())
        } else {
            Err("image not found on screen".into())
        }
    } else {
        Err("image not found on screen".into())
    }
}

pub fn find_image_loop_and_move_left_click(
    template_path: &str,
    gui: &mut RustAutoGui,
) -> Result<(), Box<dyn std::error::Error>> {
    let screen_size = gui.get_screen_size();

    if gui
        .prepare_template_from_file(
            template_path,
            Some((0, 0, screen_size.0 as u32, screen_size.1 as u32)),
            rustautogui::MatchMode::Segmented,
        )
        .is_ok()
    {
        let found_image = gui.loop_find_image_on_screen(0.9, 10)?;
        if let Some(location) = found_image {
            let location_x = location[0].0;
            let location_y = location[0].1;

            let full_location = (location_x, location_y);
            MouseOptions::MoveTo(full_location, 0.1).do_it(gui)?;
            MouseOptions::LeftClick.do_it(gui)?;

            Ok(())
        } else {
            Err("image not found on screen".into())
        }
    } else {
        Err("image not found on screen".into())
    }
}
