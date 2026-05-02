pub enum VariableReturn {
    I8(i8),
    I32(i32),
    String(String),
}

impl PartialEq for VariableReturn {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VariableReturn::I8(a), VariableReturn::I8(b)) => a == b,
            (VariableReturn::I32(a), VariableReturn::I32(b)) => a == b,
            (VariableReturn::String(a), VariableReturn::String(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for VariableReturn {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (VariableReturn::I8(a), VariableReturn::I8(b)) => a.partial_cmp(b),
            (VariableReturn::I32(a), VariableReturn::I32(b)) => a.partial_cmp(b),
            (VariableReturn::String(a), VariableReturn::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}
