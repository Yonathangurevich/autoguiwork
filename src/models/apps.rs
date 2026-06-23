use std::process::Command;

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
