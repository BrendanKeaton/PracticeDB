use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Operand {
    #[default]
    EQ,
    GT,
    GTE,
    LT,
    LTE,
    NQ,
}

impl Operand {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            ">" => Some(Operand::GT),
            ">=" => Some(Operand::GTE),
            "=" | "==" => Some(Operand::EQ),
            "<" => Some(Operand::LT),
            "<=" => Some(Operand::LTE),
            "!=" | "<>" => Some(Operand::NQ),
            _ => None,
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::EQ => write!(f, "="),
            Operand::GT => write!(f, ">"),
            Operand::GTE => write!(f, ">="),
            Operand::LT => write!(f, "<"),
            Operand::LTE => write!(f, "<="),
            Operand::NQ => write!(f, "!="),
        }
    }
}
