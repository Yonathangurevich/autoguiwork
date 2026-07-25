use autoguiwork::{
    engine::context::Context,
    engine::report::{ActionStatus, RunOutcome},
    engine::storage::{list_apps, load_app},
    models::engine_error::EngineErrorKind,
    tools::cancel_watcher::spawn_escape_watcher,
};

fn main() {
    // Simple CLI dispatch: `autoWork run <name>` or `autoWork list`.
    // This is the command-line stand-in for what the Tauri UI will call later.
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    let result = match command {
        Some("run") => match args.get(2) {
            Some(name) => run_app(name),
            None => {
                eprintln!("usage: autoWork run <name>");
                std::process::exit(2);
            }
        },
        Some("list") => list_command(),
        _ => {
            eprintln!("commands:");
            eprintln!("  autoWork run <name>    run a saved automation");
            eprintln!("  autoWork list          list saved automations");
            std::process::exit(2);
        }
    };

    // One place to turn an EngineError into a clear, non-panicking message.
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

/// Load a saved automation by name and execute it, with Esc-to-cancel armed.
fn run_app(name: &str) -> Result<(), EngineErrorKind> {
    let app = load_app(name)?;
    println!("running '{}' ({} actions)...", app.name, app.actions.len());
    println!("(press Esc to stop)");

    let mut gui = rustautogui::RustAutoGui::new(false).map_err(EngineErrorKind::AutoGui)?;

    // Build the Context here so we can grab its cancel handle and start the
    // Esc watcher BEFORE the run begins — the watcher and the runner share the
    // same flag.
    let mut ctx = Context::new();
    spawn_escape_watcher(ctx.cancel_handle());

    // execute() now returns a full RunReport — it never "errors", a failed run
    // is just a report with a Failed outcome. Print the log line by line.
    let report = app.execute(&mut gui, &mut ctx);
    for r in &report.results {
        let status = match &r.status {
            ActionStatus::Ok => "ok".to_string(),
            ActionStatus::Failed(msg) => format!("FAIL  {msg}"),
            ActionStatus::Skipped => "skipped".to_string(),
        };
        println!("  [{}] {:<28} {:>6}ms  {}", r.index, r.name, r.duration_ms, status);
    }

    match report.outcome {
        RunOutcome::Completed => {
            println!("done.");
            Ok(())
        }
        RunOutcome::Failed { at_index } => {
            eprintln!("run failed at action [{at_index}].");
            std::process::exit(1);
        }
        RunOutcome::Cancelled { at_index } => {
            eprintln!("run cancelled at action [{at_index}].");
            std::process::exit(1);
        }
    }
}

/// Print every saved automation (the CLI version of the future UI's card list).
fn list_command() -> Result<(), EngineErrorKind> {
    let names = list_apps()?;
    if names.is_empty() {
        println!("no automations saved yet.");
    } else {
        println!("saved automations:");
        for name in names {
            println!("  {name}");
        }
    }
    Ok(())
}

/* fn kobi_container_auto(gui: &mut RustAutoGui) -> Result<(), Box<dyn std::error::Error>> {
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

    // find_image_loop_and_move_left_click("./assets/kobiFile.jpeg", gui)?;

    let download_location = find_image_loop("./assets/kobiDownload.jpeg", gui)?;
    MouseOptions::MoveTo(download_location, 0.1).do_it(gui)?;

    // find_image_loop_and_move_left_click("./assets/kobiToDownload.jpeg", gui)?;

    message("לחכות לסיום ההורדה")?;

    let file_name = create_naming_by_time("צפי הגעת מוצרים", "xlsx");
    let path = format!(
        "C:\\Users\\andrey\\קונטיינר 2026\\קונטיינר גיבוי\\{}",
        file_name
    );
    
    MoveFiles::move_last_download_to(path)?;

    // find_image_loop_and_move_left_click("./assets/kobiFile.jpeg", gui)?;
    // find_image_loop_and_move_left_click("./assets/kobiImport.jpeg", gui)?;
    // find_image_loop_and_move_left_click("./assets/kobiUpload.jpeg", gui)?;

    OpenApps::Explorer("C:\\Users\\andrey\\קונטיינר 2026".to_string());

    Ok(())
}
*/