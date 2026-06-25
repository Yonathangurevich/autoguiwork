use autoguiwork::{
    // models::apps::{ExplorerTools, MoveFiles},
    tools::{
        clickes::{KeyboardOptions, MouseOptions, PcParts},
        file_naming::create_naming_by_time,
        find_image::find_image,
        message_box::message,
    },
};
// use autoguiwork::tools::clickes::{MouseOptions, PcParts};
// use autoguiwork::models::apps::Googles::Search;
// use autoguiwork::tools::find_image::find_image;
// use autoguiwork::{models::apps::OpenApps, tools::message_box::message};
use rustautogui::RustAutoGui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let to = "C:\\Users\\jonatan\\Documents\\Arduino\\testing.jpeg";
    // let from = "C:\\Users\\jonatan\\Downloads\\WhatsApp Image 2026-06-25 at 08.02.13.jpeg";

    // let moves = MoveFiles::new(from.to_string(), to.to_string());

    // ExplorerTools::MoveFile(moves).run()?;
    let mut gui = rustautogui::RustAutoGui::new(false)?;

    kobi_container_auto(&mut gui)?;
    // OpenApps::Outlook.open()?;
    // OpenApps::Google(Search("s".to_string())).open()?;

    // OpenApps::Explorer(path.to_string()).open()?;

    // message("move the open window to the main monitor")?;

    // let location = find_image("./assets/textImage.png", &mut gui)?;

    // MouseOptions::MoveTo(location).do_it(&mut gui)?;

    // MouseOptions::Drag((200, 200)).do_it(&mut gui)?;

    Ok(())
}

fn kobi_container_auto(gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
    // first he open the outlook and search to open the excel file and runs the macro
    // after he done with the macro he needs to save the file and name it
    message("move the window to the center")?;

    // clicking on the קובץ in the excel
    let excel_file_location = find_image("./assets/first.jpeg", gui)?;
    MouseOptions::MoveTo(excel_file_location).do_it(gui)?;
    MouseOptions::DoubleClick.do_it(gui)?;

    // clicking on the שמור בשם
    let save_as_excel_location = find_image("./assets/save_as.jpeg", gui)?;
    MouseOptions::MoveTo(save_as_excel_location).do_it(gui)?;
    MouseOptions::DoubleClick.do_it(gui)?;

    // clicking on the עיון
    let open_to_save_location = find_image("./assets/open_to_save.jpeg", gui)?;
    MouseOptions::MoveTo(open_to_save_location).do_it(gui)?;
    MouseOptions::DoubleClick.do_it(gui)?;

    // name the file
    let file_name = create_naming_by_time("צפי הגעת מוצרים", "csv");
    KeyboardOptions::Input(file_name).do_it(gui)?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use core::error;

use autoguiwork::models::apps::OpenApps;

use super::*;

    #[test]
    fn test_double_clicking() -> Result<(), Box<dyn error::Error>> {
        let mut gui = rustautogui::RustAutoGui::new(false)?;
        let path = "C:\\Users\\jonatan\\Documents\\Arduino";
        
        message("place in the center")?;
        OpenApps::Explorer(path.to_string());
        let test_file_location = find_image("./assets/textImage.png", &mut gui)?;
        MouseOptions::MoveTo(test_file_location).do_it(&mut gui)?;
        MouseOptions::DoubleClick.do_it(&mut gui)?;
        
        Ok(())
    }
}