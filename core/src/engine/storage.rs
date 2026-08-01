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
use crate::engine::report::RunReport;
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

/// Save an automation to its folder (creating apps/<name>/ if needed).
/// The UI calls this on every meaningful edit — the JSON is the source of truth.
pub fn save_app(app: &App) -> Result<(), EngineErrorKind> {
    let dir = app_dir(&app.name)?;
    fs::create_dir_all(&dir).map_err(|source| EngineErrorKind::AppFileIo {
        path: dir.to_string_lossy().to_string(),
        source,
    })?;
    app.save(app_json_path(&app.name)?)
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

/// Find and load the automation whose Server trigger has this endpoint_id.
/// The worker uses this to map an incoming URL (/run/<endpoint_id>) to the
/// right automation. Returns None if no saved automation exposes that endpoint.
pub fn load_app_by_endpoint(endpoint_id: &str) -> Result<Option<App>, EngineErrorKind> {
    for name in list_apps()? {
        // Skip automations that fail to load rather than aborting the scan.
        let Ok(app) = load_app(&name) else { continue };
        if app.trigger.endpoint_id() == Some(endpoint_id) {
            return Ok(Some(app));
        }
    }
    Ok(None)
}

// --- run history (per-endpoint logs) --------------------------------------

/// One saved run: when it happened plus the full report. Stored so the UI can
/// show a history of endpoint calls and their errors (headless runs can't be
/// watched live).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    /// ISO-8601 UTC timestamp.
    pub at: String,
    pub report: RunReport,
}

fn runs_dir(app_name: &str) -> Result<PathBuf, EngineErrorKind> {
    Ok(app_dir(app_name)?.join("runs"))
}

/// Persist a run's report under apps/<name>/runs/<timestamp>.json.
/// Best-effort: a logging failure must never break the actual run, so callers
/// typically ignore the error.
pub fn save_run(app_name: &str, report: &RunReport) -> Result<(), EngineErrorKind> {
    let dir = runs_dir(app_name)?;
    fs::create_dir_all(&dir).map_err(|source| EngineErrorKind::AppFileIo {
        path: dir.to_string_lossy().to_string(),
        source,
    })?;

    let now = chrono::Utc::now();
    // File-name-safe timestamp; the human ISO form is stored inside the record.
    let stamp = now.format("%Y%m%dT%H%M%S%3f").to_string();
    let record = RunRecord {
        at: now.to_rfc3339(),
        report: report.clone(),
    };

    let path = dir.join(format!("{stamp}.json"));
    let json = serde_json::to_string_pretty(&record).map_err(|source| {
        EngineErrorKind::AppFileParse {
            path: path.to_string_lossy().to_string(),
            source,
        }
    })?;
    fs::write(&path, json).map_err(|source| EngineErrorKind::AppFileIo {
        path: path.to_string_lossy().to_string(),
        source,
    })
}

/// List saved runs for an automation, newest first (capped to `limit`).
pub fn list_runs(app_name: &str, limit: usize) -> Result<Vec<RunRecord>, EngineErrorKind> {
    let dir = runs_dir(app_name)?;
    if !dir.exists() {
        return Ok(vec![]);
    }

    // Collect (filename, path); filenames are sortable timestamps.
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|source| EngineErrorKind::AppFileIo {
        path: dir.to_string_lossy().to_string(),
        source,
    })?;
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            files.push((entry.file_name().to_string_lossy().to_string(), path));
        }
    }

    // Newest first (timestamp filenames sort chronologically).
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files.truncate(limit);

    let mut records = Vec::new();
    for (_, path) in files {
        // Skip any unreadable/corrupt record rather than failing the whole list.
        let Ok(text) = fs::read_to_string(&path) else { continue };
        let Ok(record) = serde_json::from_str::<RunRecord>(&text) else { continue };
        records.push(record);
    }
    Ok(records)
}
