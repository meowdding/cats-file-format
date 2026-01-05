use crate::error::{ErrorType, Result};
use std::fmt::{Display, Formatter};


pub fn validate_name(name: String, context: &EvalContext) -> Result<String> {
    if name
        .chars()
        .all(|c| c.is_ascii_graphic() && c != '/' && c != '\\')
        && name != ".."
    {
        return Ok(name);
    }
    ErrorType::InvalidEntryName(context.clone()).into()
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
