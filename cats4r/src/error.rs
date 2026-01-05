use crate::utils::EvalContext;
use std::error::Error;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Clone)]
pub enum ErrorType {
    UnknownArg,
    InvalidInput(String),
    FailedToOpenInput {
        path: String,
        error: String,
    },
    InvalidFileType,

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

impl ErrorType {
    pub fn new(self) -> CatError {
        CatError { error_type: self }
    }
}

impl Into<i32> for ErrorType {
    fn into(self) -> i32 {
        match self {
            ErrorType::UnknownArg => -1,
            ErrorType::InvalidInput(_) => -1,
            ErrorType::FailedToOpenInput { .. } => -1,
            ErrorType::InvalidFileType => -1,

            ErrorType::UnknownVersion => 1,
            ErrorType::InvalidMetadata { .. } => 2,

            ErrorType::InvalidEntryName(_) => 100,
            ErrorType::InvalidEntryData(_) => 101,
            ErrorType::InvalidEntryType(_, _) => 102,

            ErrorType::UnableToCreateDirectory(_) => 200,
            ErrorType::ErrorWritingFile { .. } => 201,
            ErrorType::ErrorReadingFile { .. } => 202,
            ErrorType::ErrorWritingMetadata { .. } => 203,
            ErrorType::ErrorReadingMetadata { .. } => 204,
        }
    }
}

impl Into<CatError> for ErrorType {
    fn into(self) -> CatError {
        self.new()
    }
}

impl<T> Into<std::result::Result<T, CatError>> for ErrorType {
    fn into(self) -> std::result::Result<T, CatError> {
        Err(self.into())
    }
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::InvalidInput(path) => {
                f.write_str("Invalid input path '")?;
                f.write_str(path)?;
                f.write_str("'")
            }
            ErrorType::FailedToOpenInput { path, error } => {
                f.write_str("Failed to read '")?;
                f.write_str(path)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            ErrorType::InvalidFileType => f.write_str("Invalid filetype"),
            ErrorType::UnknownArg => f.write_str("Unknown Argument"),

            ErrorType::UnknownVersion => f.write_str("Unknown Version"),
            ErrorType::InvalidMetadata(context) => {
                f.write_str("Invalid Metadata at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }

            ErrorType::InvalidEntryName(context) => {
                f.write_str("Invalid filename at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }
            ErrorType::InvalidEntryData(context) => {
                f.write_str("Invalid entry data at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }
            ErrorType::InvalidEntryType(context, data) => {
                f.write_str("Invalid entry type ")?;
                u8::fmt(data, f)?;
                f.write_str(" at '")?;
                context.fmt(f)?;
                f.write_str("'")
            }

            ErrorType::UnableToCreateDirectory(dir) => {
                f.write_str("Unable to create directory '")?;
                f.write_str(dir)?;
                f.write_str("'")
            }
            ErrorType::ErrorWritingFile { path, error } => {
                f.write_str("Failed to write '")?;
                f.write_str(path)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            ErrorType::ErrorReadingFile { path, error } => {
                f.write_str("Failed to read '")?;
                f.write_str(path)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            ErrorType::ErrorWritingMetadata(context, error) => {
                f.write_str("Failed to write metadata for '")?;
                context.fmt(f)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
            ErrorType::ErrorReadingMetadata(context, error) => {
                f.write_str("Failed to read metadata for '")?;
                context.fmt(f)?;
                f.write_str("' reason: ")?;
                f.write_str(error)
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct CatError {
    error_type: ErrorType,
}

impl CatError {
    pub fn exit_code(self) -> i32 {
        self.error_type.into()
    }
}

impl Error for CatError {}
impl Display for CatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.error_type.fmt(f)
    }
}

pub(crate) type Result<T> = std::result::Result<T, CatError>;
