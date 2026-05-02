use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    FROM,
    WHERE,
    AND,
    OR,
}

impl Filter {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "from" => Some(Filter::FROM),
            "where" => Some(Filter::WHERE),
            "and" => Some(Filter::AND),
            "or" => Some(Filter::OR),
            _ => None,
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Filter::FROM => write!(f, "FROM"),
            Filter::WHERE => write!(f, "WHERE"),
            Filter::AND => write!(f, "AND"),
            Filter::OR => write!(f, "OR"),
        }
    }
}
