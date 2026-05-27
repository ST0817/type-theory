use std::fmt::Display;

use crate::parser::Term;

#[derive(Clone)]
pub enum RawType {
    Unit,
    Int,
}

impl Display for RawType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Int => write!(f, "Int"),
        }
    }
}

pub fn check_term<'src>(term: &Term) -> RawType {
    match term {
        Term::Unit => RawType::Unit,
        Term::Int { value: _ } => RawType::Int,
    }
}
