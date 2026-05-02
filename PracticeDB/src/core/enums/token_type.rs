use crate::core::enums::{Command, Filter, Operand, ValueTypes};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    CMD(Command),
    OP(Operand),
    FILTER(Filter),
    VALUE(ValueTypes),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::CMD(cmd) => write!(f, "{}", cmd),
            TokenType::OP(op) => write!(f, "{}", op),
            TokenType::FILTER(filter) => write!(f, "{}", filter),
            TokenType::VALUE(val) => write!(f, "{}", val),
        }
    }
}
