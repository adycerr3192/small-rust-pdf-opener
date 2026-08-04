//! Shared error type for the PDF opener.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),

    #[error("PDF: {0}")]
    Pdf(String),

    #[error("OCR: {0}")]
    Ocr(String),

    #[error("Sign: {0}")]
    Sign(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }

    pub fn pdf(s: impl Into<String>) -> Self {
        Self::Pdf(s.into())
    }

    pub fn ocr(s: impl Into<String>) -> Self {
        Self::Ocr(s.into())
    }

    pub fn sign(s: impl Into<String>) -> Self {
        Self::Sign(s.into())
    }
}

impl From<mupdf::Error> for AppError {
    fn from(e: mupdf::Error) -> Self {
        Self::Pdf(e.to_string())
    }
}
