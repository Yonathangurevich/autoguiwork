use rustautogui::RustAutoGui;

pub fn find_image(
    template_path: &str,
    gui: &mut RustAutoGui,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let screen_size = gui.get_screen_size();

    // preparing the template image
    if let Ok(_) = gui.prepare_template_from_file(
        template_path,
        Some((0, 0, screen_size.0 as u32, screen_size.1 as u32)),
        rustautogui::MatchMode::Segmented,
    ) {
        // finding the prepared template image
        let found_location = gui.find_image_on_screen(0.9)?;
        if let Some(location) = found_location {
            let location_x = location[0].0;
            let location_y = location[0].1;

            return Ok((location_x, location_y));

            // gui.move_mouse_to_pos(location_x, location_y, 0.1)?;
        } else {
            return Err("image not found on screen".into());
        }

    } else {
        return Err("image not found on screen".into());
    }
}
