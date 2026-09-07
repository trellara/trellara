use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("source inspection failed: {0}")]
    Capture(#[from] trellara_pg_capture::CaptureError),
    #[error("failed to write output {path}: {source}")]
    WriteOutput {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to render output: {0}")]
    Render(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CheckError>;
