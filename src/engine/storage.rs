//! Where automations live on disk, and how their assets get saved.
//!
//! Layout - every automation is a self-contained folder:
//!
//!   %APPDATA%/autoguiwork/apps/
//!     kobi/
//!       automation.json          (references "assets/step_0.png")
//!       assets/
//!         step_0.png             (a pasted screenshot, stored lossless)
//!
//! The React UI never types a path: it pastes image bytes, the backend files
//! them into the right automation's assets/ folder and hands back the short
//! relative path that goes into the JSON.

use std::fs;
use std::path::PathBuf;

use crate::engine::app::App;
use crate::models::engine_error::EngineErrorKind;

/// The root folder that holds every automation: %APPDATA%/autoguiwork/apps/
pub fn apps_root() -> Result<PathBuf, EngineErrorKind> {
    let data = dirs::data_dir().ok_or(EngineErrorKind::DataFolderUnavailable)?;
    Ok(data.join("autoguiwork").join("apps"))
}

/// The folder for one automation: %APPDATA%/autoguiwork/apps/<name>/
pub fn app_dir(app_name: &str) -> Result<PathBuf, EngineErrorKind> {
    Ok(apps_root()?.join(app_name))
}

/// The automation.json file for one automation.
pub fn app_json_path(app_name: &str) -> Result<PathBuf, EngineErrorKind> {
    Ok(app_dir(app_name)?.join("automation.json"))
}

/// Load a saved automation by name (reads apps/<name>/automation.json).
/// The returned App has its base_dir set, so its image paths resolve correctly.
pub fn load_app(app_name: &str) -> Result<App, EngineErrorKind> {
    App::load(app_json_path(app_name)?)
}

/// Save raw pasted image bytes as a lossless PNG for a given automation + step,
/// and return the RELATIVE path to store in the JSON (e.g. "assets/step_2.png").
///
/// The input bytes can be in any format the clipboard produced (PNG, BMP, ...);
/// we decode them and re-encode as PNG so matching always works on exact pixels
/// — JPEG's lossy compression would hurt image-match confidence.
pub fn save_step_image(
    app_name: &str,
    step_index: u32,
    image_bytes: &[u8],
) -> Result<String, EngineErrorKind> {
    // Decode whatever came in (format auto-detected from the bytes).
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| EngineErrorKind::InvalidImageData(e.to_string()))?;

    // Make sure apps/<name>/assets/ exists.
    let assets_dir = app_dir(app_name)?.join("assets");
    fs::create_dir_all(&assets_dir).map_err(|source| EngineErrorKind::AppFileIo {
        path: assets_dir.to_string_lossy().to_string(),
        source,
    })?;

    // Write it as PNG (lossless).
    let relative = format!("assets/step_{step_index}.png");
    let full_path = app_dir(app_name)?.join(&relative);
    img.save_with_format(&full_path, image::ImageFormat::Png)
        .map_err(|e| EngineErrorKind::InvalidImageData(e.to_string()))?;

    // The JSON stores the relative form; the engine resolves it against the
    // automation's folder at run time (see App::base_dir / resolve_path).
    Ok(relative)
}

/// List the names of every saved automation (each subfolder of the apps root).
/// This is exactly what the UI will render as clickable automation cards.
pub fn list_apps() -> Result<Vec<String>, EngineErrorKind> {
    let root = apps_root()?;
    if !root.exists() {
        return Ok(vec![]); // no automations yet is not an error
    }

    let mut names = Vec::new();
    let entries = fs::read_dir(&root).map_err(|source| EngineErrorKind::AppFileIo {
        path: root.to_string_lossy().to_string(),
        source,
    })?;
    for entry in entries {
        let Ok(entry) = entry else { continue };
        if entry.path().is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(names)
}
