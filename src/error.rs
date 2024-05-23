use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Parse error")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Serialize error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("Custom error: {0}")]
    Custom(String),
}
