use std::fmt::{self, Display, Formatter};

use chumsky::{
    Parser,
    extra::Err,
    prelude::{choice, just},
};
use parsers::{Error, int};

pub enum Term {
    Int { value: usize },
    Unit,
}

impl Display for Term {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int { value } => write!(f, "{value}"),
            Self::Unit => write!(f, "()"),
        }
    }
}

fn unit_term<'src>() -> impl Parser<'src, &'src str, Term, Err<Error<'src>>> {
    just("()").map(|_| Term::Unit)
}

fn int_term<'src>() -> impl Parser<'src, &'src str, Term, Err<Error<'src>>> {
    int().map(|value| Term::Int { value })
}

pub fn term<'src>() -> impl Parser<'src, &'src str, Term, Err<Error<'src>>> {
    choice((unit_term(), int_term()))
}
