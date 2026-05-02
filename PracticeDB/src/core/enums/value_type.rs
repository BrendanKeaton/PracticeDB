use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueTypes {
    String(String),
    Number(i32),
    STAR,
    COMMA,
}

// this from_str function will always return "Some"...
// This is so that down the line, we can update the parsing to be "better",
// IE using single or double quotes as a qualifier for strings,
// Updating to allow floats, etc, without having to change logic elsewhere (in utils.rs)

impl ValueTypes {
    pub fn from_str(s: &str) -> Option<Self> {
        if s == "*" {
            Some(ValueTypes::STAR)
        } else if s == "," {
            Some(ValueTypes::COMMA)
        } else if let Ok(num) = s.parse::<i32>() {
            Some(ValueTypes::Number(num))
        } else {
            Some(ValueTypes::String(s.to_string()))
        }
    }
}

impl ValueTypes {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ValueTypes::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl fmt::Display for ValueTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueTypes::String(s) => write!(f, "\"{}\"", s),
            ValueTypes::Number(n) => write!(f, "{}", n),
            ValueTypes::STAR => write!(f, "*"),
            ValueTypes::COMMA => write!(f, ","),
        }
    }
}
