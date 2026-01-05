use crate::utils::EvalContext;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum CatError {
    UnknownArg,
    InvalidInput(String),
    FailedToOpenInput {
        path: String,
        error: String,
    },
    InvalidFileType,

    FailedToCompressData(EvalContext, String),
    InvalidMetadata(EvalContext),
    UnknownVersion,

    InvalidEntryName(EvalContext),
    InvalidEntryData(EvalContext),
    InvalidEntryType(EvalContext, u8),

    UnableToCreateDirectory(String),
    ErrorWritingFile {
        path: String,
        error: String,
    },
    ErrorReadingFile {
        path: String,
        error: String,
    },
    ErrorWritingMetadata(EvalContext, String),
    ErrorReadingMetadata(EvalContext, String),
}

impl Into<i32> for CatError {
    fn into(self) -> i32 {
        match self {
            CatError::UnknownArg => -1,
            CatError::InvalidInput(_) => -1,
            CatError::FailedToOpenInput { .. } => -1,
            CatError::InvalidFileType => -1,

            CatError::UnknownVersion => 1,
            CatError::InvalidMetadata { .. } => 2,
            CatError::FailedToCompressData { .. } => -2,

            CatError::InvalidEntryName(_) => 100,
            CatError::InvalidEntryData(_) => 101,
            CatError::InvalidEntryType(_, _) => 102,

            CatError::UnableToCreateDirectory(_) => 200,
            CatError::ErrorWritingFile { .. } => 201,
            CatError::ErrorReadingFile { .. } => 202,
            CatError::ErrorWritingMetadata { .. } => 203,
            CatError::ErrorReadingMetadata { .. } => 204,
        }
    }
}

impl<T> Into<std::result::Result<T, CatError>> for CatError {
    fn into(self) -> std::result::Result<T, CatError> {
        Err(self.into())
    }
}

impl Display for CatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CatError::InvalidInput(path) => {
                f.write_str("Invalid input path '")?;
                f.write_str(path)?;
                f.write_str("'")
            }
            CatError::FailedToOpenInput { path, error } => {
                f.write_str("Failed to read '")?;
                f.write_str(path)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            CatError::InvalidFileType => f.write_str("Invalid filetype"),
            CatError::UnknownArg => f.write_str("Unknown Argument"),

            CatError::UnknownVersion => f.write_str("Unknown Version"),
            CatError::InvalidMetadata(context) => {
                f.write_str("Invalid Metadata at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }
            CatError::FailedToCompressData(context, error) => {
                f.write_str("Failed to compress data for '")?;
                context.fmt(f)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }

            CatError::InvalidEntryName(context) => {
                f.write_str("Invalid filename at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }
            CatError::InvalidEntryData(context) => {
                f.write_str("Invalid entry data at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }
            CatError::InvalidEntryType(context, data) => {
                f.write_str("Invalid entry type ")?;
                u8::fmt(data, f)?;
                f.write_str(" at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }

            CatError::UnableToCreateDirectory(dir) => {
                f.write_str("Unable to create directory '")?;
                f.write_str(dir)?;
                f.write_str("'")
            }
            CatError::ErrorWritingFile { path, error } => {
                f.write_str("Failed to write '")?;
                f.write_str(path)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            CatError::ErrorReadingFile { path, error } => {
                f.write_str("Failed to read '")?;
                f.write_str(path)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            CatError::ErrorWritingMetadata(context, error) => {
                f.write_str("Failed to write metadata for '")?;
                context.fmt(f)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            CatError::ErrorReadingMetadata(context, error) => {
                f.write_str("Failed to read metadata for '")?;
                context.fmt(f)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
        }
    }
}

impl Error for CatError {}

pub type Result<T> = std::result::Result<T, CatError>;