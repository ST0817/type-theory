use std::fmt::{self, Display, Formatter};

use chumsky::{
    Parser,
    extra::Err,
    prelude::{Recursive, choice, just},
    span::{SimpleSpan, SpanWrap, Spanned},
    text::ascii::keyword,
};
use parsers::{Error, Name, int, name};

#[derive(Clone, PartialEq)]
pub enum Type {
    Unit,
    Int,
    Fun {
        param_type: Box<Self>,
        body_type: Box<Self>,
    },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Int => write!(f, "Int",),
            Self::Fun {
                param_type,
                body_type,
            } => match param_type.as_ref() {
                Self::Fun { .. } => write!(f, "({param_type}) → {body_type}"),
                _ => write!(f, "{param_type} → {body_type}"),
            },
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
    App {
        callee: Spanned<Box<Self>>,
        arg: Spanned<Box<Self>>,
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
            Self::App { callee, arg } => match callee.as_ref() {
                Self::Lam { .. } => write!(f, "({}) {}", callee.inner, arg.inner),
                _ => write!(f, "{} {}", callee.inner, arg.inner),
            },
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
            param_type: Box::new(param),
            body_type: Box::new(body),
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

fn app_term<'src>(
    term: impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    term.clone()
        .spanned()
        .padded()
        .foldl(term.spanned().padded().repeated(), |callee, arg| {
            let span: SimpleSpan = (callee.span.start..arg.span.end).into();
            Term::App {
                callee: Box::new(callee.inner).with_span(callee.span),
                arg: Box::new(arg.inner).with_span(arg.span),
            }
            .with_span(span)
        })
        .map(|spanned| spanned.inner)
}

pub fn term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> {
    let mut term = Recursive::declare();
    term.define({
        let atom = choice((
            unit_term(),
            int_term(),
            lam_term(term.clone()),
            var_term(),
            term.clone().delimited_by(just('(').padded(), just(')')),
        ));
        app_term(atom)
    });
    term
}
