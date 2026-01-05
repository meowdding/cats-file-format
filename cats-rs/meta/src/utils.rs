use std::error::Error;
use crate::error::{CatError, Result};
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

pub fn validate_name(name: String, context: &EvalContext) -> Result<String> {
    if name
        .chars()
        .all(|c| c.is_ascii_graphic() && c != '/' && c != '\\')
        && name != ".."
        && name.len() != 0
    {
        return Ok(name);
    }
    CatError::InvalidEntryName(context.clone()).into()
}

#[derive(Debug)]
pub struct EvalContext {
    path: String,
    parent: Option<Box<EvalContext>>,
}

impl EvalContext {
    #[inline]
    #[must_use]
    pub const fn new(path: String) -> Self {
        EvalContext { path, parent: None }
    }

    pub fn push(&self, path: String) -> Self {
        EvalContext {
            path,
            parent: Some(Box::new(self.clone())),
        }
    }
}

impl Clone for EvalContext {
    fn clone(&self) -> Self {
        EvalContext {
            path: self.path.clone(),
            parent: match &self.parent {
                Some(parent) => Some(parent.clone()),
                None => None
            }
        }
    }
}

impl Display for EvalContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.parent {
            Some(parent) => {
                Display::fmt(&parent, f)?;
                f.write_str("/")?;
            }
            _ => {}
        }
        f.write_str(&self.path)
    }
}



pub fn write_u32(value: &u32, buffer: &mut impl Write) -> std::result::Result<(), std::io::Error> {
    buffer.write_all(&u32::to_be_bytes(*value))
}

pub fn write_u16(value: &u16, buffer: &mut impl Write) -> std::result::Result<(), std::io::Error> {
    buffer.write_all(&u16::to_be_bytes(*value))
}

pub fn write_string(
    string: &String,
    buffer: &mut impl Write,
) -> std::result::Result<(), std::io::Error> {
    let bytes = string.as_bytes();
    let length = bytes.len();
    buffer.write_all(&[length as u8])?;
    buffer.write_all(bytes)
}

pub fn read_string(buffer: &mut impl Read) -> std::result::Result<String, std::io::Error> {
    let mut size = [0u8; 1];
    buffer.read_exact(&mut size)?;
    let mut string = &mut Vec::with_capacity(size[0] as usize);
    buffer.take(size[0] as u64).read_to_end(&mut string)?;
    Ok(String::from_utf8_lossy(string).to_string())
}

pub fn read_u8(buffer: &mut impl Read) -> std::result::Result<u8, std::io::Error> {
    let mut number = [0u8; 1];
    buffer.read_exact(&mut number)?;
    Ok(number[0])
}

pub fn read_u32(buffer: &mut impl Read) -> std::result::Result<u32, std::io::Error> {
    let mut number = [0u8; 4];
    buffer.read_exact(&mut number)?;
    Ok(u32::from_be_bytes(number))
}

pub fn read_u16(buffer: &mut impl Read) -> std::result::Result<u16, std::io::Error> {
    let mut number = [0u8; 2];
    buffer.read_exact(&mut number)?;
    Ok(u16::from_be_bytes(number))
}

pub fn wrap_context<T, E>(
    result: std::result::Result<T, E>,
    context: EvalContext,
    converter: fn(EvalContext, String) -> CatError,
) -> Result<T>
where
    E: Error,
{
    result.map_err(|err| converter(context.clone(), err.to_string()).into())
}
