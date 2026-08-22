//! Tauri commands — the bridge between the React UI and the engine.
//!
//! Each #[tauri::command] is callable from TypeScript via invoke("name", args).
//! They stay THIN: parse/convert, call into `autoguiwork` (the core engine),
//! and turn engine errors into strings the frontend can display.

use std::sync::Mutex;

use autoguiwork::engine::app::App;
use autoguiwork::engine::context::Context;
use autoguiwork::engine::report::RunReport;
use autoguiwork::engine::server::serve_in_background;
use autoguiwork::engine::storage::{
    RunRecord, app_dir, list_apps, list_runs, load_app, load_canvas, save_app, save_canvas,
    save_step_image,
};
use autoguiwork::engine::trigger::{InputSpec, Trigger, generate_endpoint_id};
use autoguiwork::tools::cancel_watcher::spawn_escape_watcher;

/// Tracks whether the shared worker server is running and on which port.
/// One server for the whole app; each exposed automation is a route on it.
#[derive(Default)]
pub struct ServerState {
    pub inner: Mutex<Option<u16>>, // Some(port) when running
}

const DEFAULT_PORT: u16 = 8787;

/// List every saved automation's name — the UI renders these as cards.
#[tauri::command]
pub fn list_automations() -> Result<Vec<String>, String> {
    list_apps().map_err(|e| e.to_string())
}

/// Create a brand-new, empty automation (Manual trigger, no steps) and save it.
/// Rejects empty or duplicate names so the Home page can create then navigate.
#[tauri::command]
pub fn create_automation(name: String) -> Result<App, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("please give the automation a name".into());
    }
    // Reject if a folder with this name already exists.
    let dir = app_dir(&name).map_err(|e| e.to_string())?;
    if dir.exists() {
        return Err(format!("an automation named '{name}' already exists"));
    }

    let app = App::new(name);
    save_app(&app).map_err(|e| e.to_string())?;
    Ok(app)
}

/// Load one automation's full definition (so the dashboard can rebuild its UI
/// from the JSON — the source of truth).
#[tauri::command]
pub fn load_automation(name: String) -> Result<App, String> {
    load_app(&name).map_err(|e| e.to_string())
}

/// Persist an automation. The UI calls this on meaningful edits (add/reorder a
/// step, finish editing a field). Writing the JSON IS the save.
#[tauri::command]
pub fn save_automation(app: App) -> Result<(), String> {
    save_app(&app).map_err(|e| e.to_string())
}

/// Save a pasted/loaded image for a FindImage step as a lossless PNG in the
/// automation's assets/ folder, and return the relative path to store in the
/// step's image_path. `bytes` are the raw image bytes (any format — clipboard
/// PNG, etc.); the engine re-encodes to PNG so matching stays pixel-exact.
#[tauri::command]
pub fn save_image(app_name: String, step_index: u32, bytes: Vec<u8>) -> Result<String, String> {
    save_step_image(&app_name, step_index, &bytes).map_err(|e| e.to_string())
}

/// Read a saved template image (by the step's relative path) and return it as a
/// data: URL, so the UI can preview it after navigating away or reopening — the
/// webview can't load files off disk directly.
#[tauri::command]
pub fn read_image(app_name: String, relative_path: String) -> Result<String, String> {
    use base64::Engine as _;

    let path = app_dir(&app_name)
        .map_err(|e| e.to_string())?
        .join(&relative_path);
    let bytes = std::fs::read(&path).map_err(|e| format!("could not read image: {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

/// Run an automation once (the Test button) and return its full RunReport.
///
/// The run is BLOCKING and takes over the real mouse/keyboard, so it must not
/// run on Tauri's async pool — spawn_blocking moves it to a dedicated blocking
/// thread, keeping the UI responsive. Esc-to-cancel is armed as in the CLI.
#[tauri::command]
pub async fn run_automation(name: String) -> Result<RunReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let app = load_app(&name).map_err(|e| e.to_string())?;

        let mut gui =
            rustautogui::RustAutoGui::new(false).map_err(|e| e.to_string())?;

        // Build the Context, arm Esc-cancel before running (shared flag).
        let mut ctx = Context::new();
        spawn_escape_watcher(ctx.cancel_handle());

        Ok(app.execute(&mut gui, &mut ctx))
    })
    .await
    .map_err(|e| format!("run task failed: {e}"))?
}

// --- worker server ---------------------------------------------------------

/// Ensure the shared worker server is running (idempotent). Returns the port.
async fn ensure_server_running(state: &ServerState) -> Result<u16, String> {
    // Already running? Return its port.
    if let Some(port) = *state.inner.lock().unwrap() {
        return Ok(port);
    }
    // Bind + spawn the accept loop; a port conflict surfaces here.
    serve_in_background(DEFAULT_PORT)
        .await
        .map_err(|e| e.to_string())?;
    *state.inner.lock().unwrap() = Some(DEFAULT_PORT);
    Ok(DEFAULT_PORT)
}

/// Is the worker server running? (for the UI status indicator)
#[tauri::command]
pub fn server_status(state: tauri::State<'_, ServerState>) -> Option<u16> {
    *state.inner.lock().unwrap()
}

/// Expose an automation as an API endpoint (the "set to production" action):
/// give it a Server trigger with a stable endpoint_id + declared inputs, save
/// it, make sure the server is running, and return the full URL to call.
#[tauri::command]
pub async fn expose_as_server(
    name: String,
    inputs: Vec<InputSpec>,
    state: tauri::State<'_, ServerState>,
) -> Result<String, String> {
    let mut app = load_app(&name).map_err(|e| e.to_string())?;

    // Reuse an existing endpoint_id if already exposed, so the URL stays stable.
    let endpoint_id = match app.trigger.endpoint_id() {
        Some(id) => id.to_string(),
        None => generate_endpoint_id(&app.name),
    };
    app.trigger = Trigger::Server {
        endpoint_id: endpoint_id.clone(),
        inputs,
    };
    save_app(&app).map_err(|e| e.to_string())?;

    let port = ensure_server_running(&state).await?;
    Ok(format!("http://127.0.0.1:{port}/run/{endpoint_id}"))
}

/// Turn an automation back into a Manual (unexposed) one.
#[tauri::command]
pub fn unexpose_server(name: String) -> Result<(), String> {
    let mut app = load_app(&name).map_err(|e| e.to_string())?;
    app.trigger = Trigger::Manual;
    save_app(&app).map_err(|e| e.to_string())
}

/// List recent runs (newest first) for an automation — the endpoint call
/// history + errors shown in the Runs tab.
#[tauri::command]
pub fn automation_runs(name: String) -> Result<Vec<RunRecord>, String> {
    list_runs(&name, 50).map_err(|e| e.to_string())
}

// --- canvas layout (UI only; the engine never reads it) --------------------

/// Save the visual editor's layout (node positions + parked nodes).
/// Opaque JSON: the backend stores it verbatim and never interprets it.
#[tauri::command]
pub fn save_canvas_layout(name: String, layout: serde_json::Value) -> Result<(), String> {
    save_canvas(&name, &layout).map_err(|e| e.to_string())
}

/// Load the canvas layout, or null if this automation has no saved layout yet.
#[tauri::command]
pub fn load_canvas_layout(name: String) -> Result<Option<serde_json::Value>, String> {
    load_canvas(&name).map_err(|e| e.to_string())
}
