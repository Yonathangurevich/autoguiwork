use autoguiwork::tools::clickes::{MouseOptions, PcParts};
// use autoguiwork::models::apps::Googles::Search;
use autoguiwork::tools::find_image::find_image;
use autoguiwork::{models::apps::OpenApps, tools::message_box::message};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "C:\\Users\\jonatan\\Documents\\Arduino";

    let mut gui = rustautogui::RustAutoGui::new(false)?;

    // OpenApps::Outlook.open()?;
    // OpenApps::Google(Search("s".to_string())).open()?;

    OpenApps::Explorer(path.to_string()).open()?;

    message("move the open window to the main monitor")?;

    let location = find_image("./assets/textImage.png", &mut gui)?;

    MouseOptions::MoveTo(location).do_it(&mut gui)?;

    MouseOptions::RightClick.do_it(&mut gui)?;

    Ok(())
}
