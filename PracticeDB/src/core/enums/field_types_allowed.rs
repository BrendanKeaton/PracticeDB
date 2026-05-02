use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldTypesAllowed {
    I8,
    I32,
    String,
}

impl FieldTypesAllowed {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "i8" => Some(FieldTypesAllowed::I8),
            "i32" => Some(FieldTypesAllowed::I32),
            "string" => Some(FieldTypesAllowed::String),
            _ => None,
        }
    }
}

impl fmt::Display for FieldTypesAllowed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FieldTypesAllowed::I8 => "i8",
            FieldTypesAllowed::I32 => "i32",
            FieldTypesAllowed::String => "string",
        };
        write!(f, "{}", s)
    }
}
