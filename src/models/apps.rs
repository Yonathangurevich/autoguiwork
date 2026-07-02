use std::fs;
use std::process::Command;
extern crate dirs;

pub enum OpenApps {
    Outlook,
    Google(Googles),
    Explorer(String),
}

pub enum Googles {
    Drive,
    Chrome,
    Search(String),
}

impl OpenApps {
    pub fn open(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            OpenApps::Outlook => {
                Command::new("cmd")
                    .args(["/C", "start", "", "outlook.exe"])
                    .spawn()?;
            }
            OpenApps::Google(g) => {
                Command::new("cmd")
                    .args(["/C", "start", "", g.to_web()])
                    .spawn()?;
            }
            OpenApps::Explorer(path) => {
                Command::new("explorer").arg(path).spawn()?;
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

// tools for the filesystem
pub enum ExplorerTools {
    // moving file from - the path its now, to - the file path you want
    MoveFile(MoveFiles),
}

impl ExplorerTools {
    pub fn run(&self) -> Result<(), std::io::Error> {
        match self {
            Self::MoveFile(paths) => {
                let from = &paths.from;
                let to = &paths.to;
                // the "to" should already have the rename
                fs::rename(from, to)
            }
        }
    }
}

// ----

#[derive(Debug)]
pub struct MoveFiles {
    from: String,
    to: String,
}

impl MoveFiles {
    // will create the transfer file location based on already knowen path's
    pub fn new(from: String, to: String) -> Self {
        Self { from, to }
    }

    // will find the last downloaded file and transfer to knowen path
    pub fn find_last_downloaded(to: String) -> Option<Self> {
        let download_path = dirs::download_dir()?;
        let mut results = Vec::new();

        // checking the last created file in downloads
        for entry in fs::read_dir(&download_path).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if path.is_file() {
                let meta = fs::metadata(&path).ok()?;
                if let Ok(newest) = meta.created() {
                    results.push((file_name, newest));
                }
            }
        }

        // sorting and placeing the newest on top
        results.sort_by_key(|file| file.1);
        results.reverse();

        // getting both the path and the file name
        let download_path_string = download_path.to_string_lossy().to_string();
        let newest_file = &results[0].0;

        // building the final "from"
        let from = format!("{}\\{}", download_path_string, newest_file);

        Some(Self { from, to })
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

    #[test]
    fn test_find_last_download_and_move_to() {
        let path = "C:\\Users\\jonatan\\Documents\\Arduino\\testSheet.xlsx".to_string();
        if let Some(last) = MoveFiles::find_last_downloaded(path) {
            if let Ok(_) = ExplorerTools::MoveFile(last).run() {
                println!("done go look");
            } else {
                println!("failed");
            }
        }
    }
}
