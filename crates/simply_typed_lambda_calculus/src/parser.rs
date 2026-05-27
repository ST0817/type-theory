use std::fmt::{self, Display, Formatter};

use chumsky::{
    Parser,
    extra::Err,
    prelude::{Recursive, choice, just},
    text::ascii::keyword,
};
use parsers::{Error, Name, int, name};

#[derive(Clone)]
pub enum Type {
    Unit,
    Int,
    Fun { param: Box<Self>, body: Box<Self> },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Int => write!(f, "Int",),
            Self::Fun { param, body } => write!(f, "({param} → {body})"),
        }
    }
}

pub enum Term<'src> {
    Unit,
    Int {
        value: usize,
    },
    Lam {
        param_name: Name<'src>,
        param_type: Type,
        body: Box<Self>,
    },
    Var {
        name: Name<'src>,
    },
}

impl Display for Term<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Int { value } => write!(f, "{value}"),
            Self::Var { name } => write!(f, "{}", name.inner),
            Self::Lam {
                param_name,
                param_type,
                body,
            } => write!(f, "λ{} : {}. {}", param_name.inner, param_type, body),
        }
    }
}

fn unit_type<'src>() -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    just("()").map(|_| Type::Unit)
}

fn int_type<'src>() -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    keyword("Int").map(|_| Type::Int)
}

fn fun_type<'src>(
    ty: impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    ty.clone().padded().foldl(
        just("->").padded().ignore_then(ty).padded().repeated(),
        |param, body| Type::Fun {
            param: Box::new(param),
            body: Box::new(body),
        },
    )
}

fn ty<'src>() -> impl Parser<'src, &'src str, Type, Err<Error<'src>>> + Clone {
    let atom = choice((unit_type(), int_type()));
    fun_type(atom)
}

fn unit_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    just("()").map(|_| Term::Unit)
}

fn int_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    int().map(|value| Term::Int { value })
}

fn lam_term<'src>(
    term: impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("lam")
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(':'))
        .padded()
        .then(ty())
        .padded()
        .then_ignore(just('.'))
        .padded()
        .then(term.map(Box::new))
        .map(|((param_name, param_type), body)| Term::Lam {
            param_name,
            param_type,
            body,
        })
}

fn var_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    name().map(|name| Term::Var { name })
}

pub fn term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> {
    let mut term = Recursive::declare();
    term.define(choice((
        unit_term(),
        int_term(),
        lam_term(term.clone()),
        var_term(),
    )));
    term
}
