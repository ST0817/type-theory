use std::fmt::{self, Display, Formatter};

pub enum Expr {
    Unit,
    UnitType,
    Nat { value: usize },
    NatType,
    Sort { level: usize },
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::UnitType => write!(f, "Unit"),
            Self::Nat { value } => write!(f, "{value}"),
            Self::NatType => write!(f, "Nat"),
            Self::Sort { level } => write!(f, "Sort {level}"),
        }
    }
}
