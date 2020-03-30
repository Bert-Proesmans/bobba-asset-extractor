use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum ExtractorError {
    Argument(clap::Error),
    Runtime(crate::runtime::Error),
    Other(String),
}

impl fmt::Display for ExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ExtractorError::Argument(ref clap) => write!(f, "Error during argument parsing: {}", clap),
            ExtractorError::Runtime(ref runtime) => write!(f, "Runtime error: {}", runtime),
            ExtractorError::Other(ref string) => write!(f, "Constraint error: {}", string),
        }
    }
}

impl Error for ExtractorError {}