mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::ServerState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_automations,
            commands::create_automation,
            commands::load_automation,
            commands::save_automation,
            commands::save_image,
            commands::read_image,
            commands::run_automation,
            commands::server_status,
            commands::expose_as_server,
            commands::unexpose_server,
            commands::automation_runs,
            commands::save_canvas_layout,
            commands::load_canvas_layout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
