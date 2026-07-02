#![windows_subsystem = "windows"]

use autoguiwork::{
    models::apps::{
        ExplorerTools::{self},
        Googles::Search,
        MoveFiles, OpenApps,
    },
    tools::{
        clickes::{KeyboardOptions, MouseOptions, PcParts},
        file_naming::create_naming_by_time,
        find_image::{find_image_loop, find_image_loop_and_move_left_click},
        message_box::message,
    },
};
use rustautogui::RustAutoGui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gui = rustautogui::RustAutoGui::new(false)?;
    kobi_container_auto(&mut gui)?;
    Ok(())
}

fn kobi_container_auto(gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
    // first he open the outlook and search to open the excel file and runs the macro
    // after he done with the macro he needs to save the file and name it
    message("move the window to the center")?;

    eprintln!("will find the file to save");

    // clicking on the קובץ in the excel
    let excel_file_location = find_image_loop("./assets/first.jpeg", gui)?;
    MouseOptions::MoveTo(excel_file_location, 0.1).do_it(gui)?;
    MouseOptions::DoubleClick.do_it(gui)?;

    eprintln!("clicked on the file and trying to find the save as");

    // clicking on the שמור בשם
    let save_as_excel_location = find_image_loop("./assets/save_as.jpeg", gui)?;
    MouseOptions::MoveTo(save_as_excel_location, 0.1).do_it(gui)?;
    MouseOptions::DoubleClick.do_it(gui)?;

    eprintln!("clicked on the save as and trying to find the next");

    // clicking on the עיון
    let open_to_save_location = find_image_loop("./assets/open_to_save.jpeg", gui)?;
    MouseOptions::MoveTo(open_to_save_location, 0.1).do_it(gui)?;
    MouseOptions::DoubleClick.do_it(gui)?;

    // name the file - hebrew is a problem now
    let file_name = create_naming_by_time("", "csv");
    KeyboardOptions::Input(file_name).do_it(gui)?;

    message("תשמור את הקובץ או שנה את השם")?;

    OpenApps::Google(Search("https://docs.google.com/spreadsheets/d/1nIFFn2pIVWOulNuoIkZufzzLJoDeISMGU8IDgPr9NMc/edit?gid=0#gid=0".to_string())).open()?;

    message("center")?;

    find_image_loop_and_move_left_click("./assets/kobiFile.jpeg", gui)?;

    let download_location = find_image_loop("./assets/kobiDownload.jpeg", gui)?;
    MouseOptions::MoveTo(download_location, 0.1).do_it(gui)?;

    find_image_loop_and_move_left_click("./assets/kobiToDownload.jpeg", gui)?;

    message("לחכות לסיום ההורדה")?;

    let file_name = create_naming_by_time("צפי הגעת מוצרים", "xlsx");
    let path = format!(
        "C:\\Users\\andrey\\קונטיינר 2026\\קונטיינר גיבוי\\{}",
        file_name
    );
    if let Some(last) = MoveFiles::find_last_downloaded(path) {
        ExplorerTools::MoveFile(last).run()?;
    }

    find_image_loop_and_move_left_click("./assets/kobiFile.jpeg", gui)?;
    find_image_loop_and_move_left_click("./assets/kobiImport.jpeg", gui)?;
    find_image_loop_and_move_left_click("./assets/kobiUpload.jpeg", gui)?;

    OpenApps::Explorer("C:\\Users\\andrey\\קונטיינר 2026".to_string());

    Ok(())
}
