use std::fs;
use std::process::Command;

use crate::models::engine_error::EngineErrorKind::{self, DownloadsFolderUnavailable};
extern crate dirs;

// -- fucntions for opening apps.

#[derive(Debug)]
pub enum OpenApps {
    Outlook,
    Google(Googles),
    Explorer(String),
    RandomApp(String),
}

#[derive(Debug)]
pub enum Googles {
    Drive,
    Chrome,
    Search(String),
}

impl OpenApps {
    pub fn open(&self) -> Result<(), EngineErrorKind> {
        match self {
            OpenApps::Outlook => {
                Command::new("cmd")
                    .args(["/C", "start", "", "outlook.exe"])
                    .spawn()
                    .map_err(|e| EngineErrorKind::ProcessSpawn {
                        command: OpenApps::Outlook.to_string(),
                        source: e,
                    })?;
            }
            OpenApps::Google(g) => {
                Command::new("cmd")
                    .args(["/C", "start", "", g.to_web()])
                    .spawn()
                    .map_err(|e| EngineErrorKind::ProcessSpawn {
                        command: g.to_string(),
                        source: e,
                    })?;
            }
            OpenApps::Explorer(path) => {
                Command::new("explorer").arg(path).spawn().map_err(|e| {
                    EngineErrorKind::ProcessSpawn {
                        command: OpenApps::Explorer(path.clone()).to_string(),
                        source: e,
                    }
                })?;
            }
            OpenApps::RandomApp(app) => {
                Command::new("cmd")
                    .args(["/C", "start", "", app])
                    .spawn()
                    .map_err(|e| EngineErrorKind::ProcessSpawn {
                        command: OpenApps::RandomApp(app.clone()).to_string(),
                        source: e,
                    })?;
            }
        };

        Ok(())
    }
}

impl Googles {
    fn to_web(&self) -> &str {
        match self {
            Googles::Drive => "https://drive.google.com",
            Googles::Chrome => "https://www.google.com",
            Googles::Search(link) => link,
        }
    }
}

// ---- functions for using the fileSystem (finding last downlaod file in the donwloads)

#[derive(Debug)]
pub struct MoveFiles;

impl MoveFiles {
    // will create the transfer file location based on already knowen path's
    pub fn new(from: String, to: String) -> Result<(), EngineErrorKind> {
        fs::rename(&from, &to).map_err(|source| EngineErrorKind::FileMoveFailed { 
            from, to, source 
        })
    }

    // will find the last downloaded file and transfer to knowen path
    pub fn move_last_download_to(to: String) -> Result<(), EngineErrorKind> {
        
        let download_path = match dirs::download_dir() {
            Some(p) => p.to_string_lossy().to_string(),
            None => return Err(DownloadsFolderUnavailable)
        };
        
        let mut results = Vec::new();

        // checking the last created file in downloads
        let entries = fs::read_dir(&download_path)
            .map_err(|source| EngineErrorKind::FileSystem { 
                path: download_path.clone(), source 
            })?;
        
        for entry in entries {
            
            let entry = entry.map_err(|source| EngineErrorKind::FileSystem { 
                path: download_path.clone(), source 
            })?;

            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if path.is_file() {
                let meta = fs::metadata(&path).map_err(|source| EngineErrorKind::FileSystem { 
                    path: path.to_string_lossy().to_string(), source 
                })?;
                if let Ok(newest) = meta.created() {
                    results.push((file_name, newest));
                }
            }
        }

        // sorting and placeing the newest on top
        results.sort_by_key(|file| file.1);
        results.reverse();

        if results.is_empty() {
            return Err(EngineErrorKind::DownloadsEmpty);
        };
        
        // getting the newest file.
        let newest_file = &results[0].0;

        // building the final "from"
        let from = format!("{}\\{}",download_path, newest_file);

        // moving the file from the downloads to the target path.
        fs::rename(&from, &to).map_err(|source| EngineErrorKind::FileMoveFailed { 
            from, to, source 
        })
        
    }
}

impl std::fmt::Display for OpenApps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            OpenApps::Explorer(path) => format!("Explorer: {path}"),
            OpenApps::Google(e) => format!("Google - {}", e.to_string()),
            OpenApps::Outlook => format!("Outlook"),
            OpenApps::RandomApp(link) => format!("RandomApp: {link}"),
        };
        return write!(f, "{name}");
    }
}

impl std::fmt::Display for Googles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Googles::Chrome => format!("Chrome"),
            Googles::Drive => format!("Drive"),
            Googles::Search(url) => format!("Search: {url}"),
        };

        return write!(f, "{name}");
    }
}

#[cfg(test)]
mod tests {
    use crate::models::apps::*;

    #[test]
    fn test_app_opening_with_explorer_path() -> Result<(), Box<dyn std::error::Error>> {
        let path = "C:\\Users\\jonatan\\Documents\\Arduino";
        assert!(
            OpenApps::Explorer(path.to_string()).open().is_ok(),
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

    #[test]
    fn test_random_app_opening() {
        let random_link = "priority:priform@CINVOICES::.:tabula.ini:1".to_string();
        let _another_link = "priority:priform@AINVOICES::.:tabula.ini:1".to_string();
        let _ = OpenApps::RandomApp(random_link).open().unwrap();
    }
}
