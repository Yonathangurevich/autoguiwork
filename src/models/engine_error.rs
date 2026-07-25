use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineErrorKind {
    #[error("automation input failed: {0}")]
    AutoGui(#[from] rustautogui::errors::AutoGuiError),

    #[error("failed to show message box: {0}")]
    MessageBox(#[from] msgbox::MsgBoxError),

    #[error("filesystem operation failed on '{path}': {source}")]
    FileSystem {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to move file from '{from}' to '{to}': {source}")]
    FileMoveFailed {
        from: String,
        to: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to launch process '{command}': {source}")]
    ProcessSpawn {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("could not load template image '{path}': {reason}")]
    InvalidTemplate { path: String, reason: String },

    #[error("image '{path}' was not found on screen after {waited_ms}ms")]
    ImageNotFound { path: String, waited_ms: f64 },

    #[error("window with title '{title}' did not appear after {waited_ms}ms")]
    WindowNotFound { title: String, waited_ms: f64 },

    #[error("no files found in the Downloads folder")]
    DownloadsEmpty,

    #[error("a download did not finish within {waited_ms}ms (newest file: '{newest}')")]
    DownloadTimedOut { newest: String, waited_ms: f64 },

    #[error("could not locate the Downloads folder on this system")]
    DownloadsFolderUnavailable,

    #[error("action referenced variable '{name}' which was never set")]
    UnknownVariable { name: String },

    #[error("variable '{name}' has type '{actual}' but action expected '{expected}'")]
    VariableTypeMismatch {
        name: String,
        expected: String,
        actual: String,
    },

    #[error("execution was cancelled by user")]
    Cancelled,
}

#[derive(Debug)]
pub struct EngineError {
    pub action_index: u32,
    pub action_name: String,
    pub kind: EngineErrorKind,
}
