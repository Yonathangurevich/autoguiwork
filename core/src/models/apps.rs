use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::engine::context::{Context, TextArg};
use crate::models::engine_error::EngineErrorKind::{self, DownloadsFolderUnavailable};
use crate::tools::find_image::sleep_for_f64;
extern crate dirs;

// Temp extensions browsers use WHILE a download is in progress. When the file
// still ends in one of these, the download isn't finished yet.
const IN_PROGRESS_EXTENSIONS: [&str; 3] = ["crdownload", "part", "tmp"];

// -- fucntions for opening apps.

#[derive(Debug, Serialize, Deserialize)]
pub enum OpenApps {
    Outlook,
    Google(Googles),
    // Explorer path and custom command can come from args (TextArg), so e.g.
    // opening a folder or launching a link the caller passed in works.
    Explorer(TextArg),
    RandomApp(TextArg),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Googles {
    Drive,
    Chrome,
    // The URL can be a fixed link or an arg, e.g. Search(Var("link")).
    Search(TextArg),
}

impl OpenApps {
    // Takes &Context so TextArg fields (paths, URLs, commands) resolve to the
    // caller's args at run time.
    pub fn open(&self, ctx: &Context) -> Result<(), EngineErrorKind> {
        match self {
            OpenApps::Outlook => {
                Command::new("cmd")
                    .args(["/C", "start", "", "outlook.exe"])
                    .spawn()
                    .map_err(|e| EngineErrorKind::ProcessSpawn {
                        command: "Outlook".to_string(),
                        source: e,
                    })?;
            }
            OpenApps::Google(g) => {
                let url = g.to_web(ctx)?;
                Command::new("cmd")
                    .args(["/C", "start", "", &url])
                    .spawn()
                    .map_err(|e| EngineErrorKind::ProcessSpawn {
                        command: url,
                        source: e,
                    })?;
            }
            OpenApps::Explorer(path) => {
                let path = path.resolve(ctx)?;
                Command::new("explorer").arg(&path).spawn().map_err(|e| {
                    EngineErrorKind::ProcessSpawn {
                        command: format!("Explorer: {path}"),
                        source: e,
                    }
                })?;
            }
            OpenApps::RandomApp(app) => {
                let cmd = app.resolve(ctx)?;
                Command::new("cmd")
                    .args(["/C", "start", "", &cmd])
                    .spawn()
                    .map_err(|e| EngineErrorKind::ProcessSpawn {
                        command: cmd,
                        source: e,
                    })?;
            }
        };

        Ok(())
    }
}

impl Googles {
    fn to_web(&self, ctx: &Context) -> Result<String, EngineErrorKind> {
        Ok(match self {
            Googles::Drive => "https://drive.google.com".to_string(),
            Googles::Chrome => "https://www.google.com".to_string(),
            Googles::Search(link) => link.resolve(ctx)?,
        })
    }
}

// ---- functions for using the fileSystem (finding last downlaod file in the donwloads)

#[derive(Debug)]
pub struct MoveFiles;

impl MoveFiles {
    /// Move a file between two known paths (no Downloads lookup — you supply both).
    pub fn move_between(from: String, to: String) -> Result<(), EngineErrorKind> {
        fs::rename(&from, &to).map_err(|source| EngineErrorKind::FileMoveFailed {
            from,
            to,
            source,
        })
    }

    /// Find the newest file (by modified time) in the Downloads folder.
    /// Returns its full path. Errors if the folder is missing or empty.
    ///
    /// Note we use *modified* time, not *created* time: Windows can preserve an
    /// old creation timestamp when you re-download a file (file system tunneling),
    /// which would make a fresh download look old. Modified time is reliable.
    fn newest_in_downloads() -> Result<PathBuf, EngineErrorKind> {
        let download_dir = dirs::download_dir().ok_or(DownloadsFolderUnavailable)?;
        let download_str = download_dir.to_string_lossy().to_string();

        let entries = fs::read_dir(&download_dir).map_err(|source| EngineErrorKind::FileSystem {
            path: download_str.clone(),
            source,
        })?;

        let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;

        for entry in entries {
            // Skip individual unreadable entries instead of aborting the whole
            // scan — one weird file shouldn't fail the entire lookup.
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Ok(meta) = fs::metadata(&path) else { continue };
            let Ok(modified) = meta.modified() else { continue };

            // keep the one with the latest modified time
            match &newest {
                Some((_, best)) if *best >= modified => {}
                _ => newest = Some((path, modified)),
            }
        }

        newest
            .map(|(path, _)| path)
            .ok_or(EngineErrorKind::DownloadsEmpty)
    }

    /// True while a path looks like an in-progress download (has a temp extension).
    fn is_in_progress(path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => IN_PROGRESS_EXTENSIONS.contains(&ext.to_lowercase().as_str()),
            None => false,
        }
    }

    /// Wait until the newest download in the folder has finished, then return
    /// its path. "Finished" is confirmed by two independent signals:
    ///   1. the newest file no longer has a temp extension (.crdownload/.part/.tmp)
    ///   2. its size is identical across two checks 500ms apart (nothing still writing)
    ///
    /// This works for tiny and huge files alike, because we wait on the actual
    /// condition rather than guessing a fixed sleep. Gives up after `timeout_ms`.
    pub fn wait_for_download_complete(timeout_ms: f64) -> Result<PathBuf, EngineErrorKind> {
        const POLL_MS: f64 = 500.0;
        let mut remaining = timeout_ms;
        let mut last_size: Option<u64> = None;

        loop {
            let newest = Self::newest_in_downloads()?;

            // Signal 1: still has a temp extension → browser is still writing it.
            if Self::is_in_progress(&newest) {
                last_size = None; // reset stability tracking; this isn't the final file yet
            } else {
                // Signal 2: has the size stopped changing?
                let size = fs::metadata(&newest)
                    .map_err(|source| EngineErrorKind::FileSystem {
                        path: newest.to_string_lossy().to_string(),
                        source,
                    })?
                    .len();

                if last_size == Some(size) {
                    // same size two checks in a row → download is done
                    return Ok(newest);
                }
                last_size = Some(size);
            }

            if remaining <= 0.0 {
                return Err(EngineErrorKind::DownloadTimedOut {
                    newest: Self::newest_in_downloads()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    waited_ms: timeout_ms,
                });
            }

            sleep_for_f64(POLL_MS / 1000.0);
            remaining -= POLL_MS;
        }
    }

    /// Wait for the current download to finish, then move it to `to`.
    /// This is the safe replacement for the old "grab newest file immediately"
    /// logic, which could move a half-downloaded `.crdownload` file.
    pub fn move_last_download_to(to: String, timeout_ms: f64) -> Result<(), EngineErrorKind> {
        let from = Self::wait_for_download_complete(timeout_ms)?;
        let from_str = from.to_string_lossy().to_string();

        fs::rename(&from, &to).map_err(|source| EngineErrorKind::FileMoveFailed {
            from: from_str,
            to,
            source,
        })
    }
}

impl std::fmt::Display for OpenApps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OpenApps::Explorer(path) => format!("Explorer: {path}"),
            OpenApps::Google(e) => format!("Google - {}", e),
            OpenApps::Outlook => "Outlook".to_string(),
            OpenApps::RandomApp(link) => format!("RandomApp: {link}"),
        };
        write!(f, "{name}")
    }
}

impl std::fmt::Display for Googles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Googles::Chrome => "Chrome".to_string(),
            Googles::Drive => "Drive".to_string(),
            Googles::Search(url) => format!("Search: {url}"),
        };

        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::context::{Context, TextArg};
    use crate::models::apps::*;

    // Actually opens an Explorer window at a hardcoded path — manual/POC test.
    #[test]
    #[ignore = "opens a real Explorer window"]
    fn test_app_opening_with_explorer_path() -> Result<(), Box<dyn std::error::Error>> {
        let path = "C:\\Users\\jonatan\\Documents\\Arduino";
        let ctx = Context::new();
        assert!(
            OpenApps::Explorer(TextArg::Literal(path.to_string()))
                .open(&ctx)
                .is_ok(),
            "failed to open explorer"
        );
        Ok(())
    }

    // #[test]
    // fn test_find_last_download_and_move_to() {
    //     let path = "C:\\Users\\jonatan\\Documents\\Arduino\\testSheet.xlsx".to_string();
    //     if let Some(last) = MoveFiles::find_last_downloaded(path) {
    //         if let Ok(_) = ExplorerTools::MoveFile(last).run() {
    //             println!("done go look");
    //         } else {
    //             println!("failed");
    //         }
    //     }
    // }

    // Launches the Priority ERP, which isn't installed on every dev machine —
    // #[ignore] so `cargo test` stays green; run explicitly with
    // `cargo test test_random_app_opening -- --ignored` on a machine that has it.
    #[test]
    #[ignore = "launches Priority ERP; not installed on all machines"]
    fn test_random_app_opening() {
        let random_link = "priority:priform@CINVOICES::.:tabula.ini:1".to_string();
        let ctx = Context::new();
        OpenApps::RandomApp(TextArg::Literal(random_link))
            .open(&ctx)
            .unwrap();
    }
}
