use thiserror::Error;

#[derive(Error, Debug)]
pub enum NavigatorError {
    #[error("ClingoError: ")]
    Clingo(#[from] clingo::ClingoError),
    #[error("Unwrapped None.")]
    None,
    #[error("IOError: ")]
    IOError(#[from] std::io::Error),
    #[error("Invalid input.")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, NavigatorError>;

