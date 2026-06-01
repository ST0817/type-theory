use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use chumsky::{
    Parser,
    extra::Err,
    prelude::{Recursive, choice, just},
    span::{self, SimpleSpan, SpanWrap},
    text::ascii::keyword,
};
use parsers::{Error, int, name};

#[derive(Clone)]
pub struct Spanned<T>(span::Spanned<T>);

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.inner == other.0.inner
    }
}

impl<T> Deref for Spanned<T> {
    type Target = span::Spanned<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<span::Spanned<T>> for Spanned<T> {
    fn from(value: span::Spanned<T>) -> Self {
        Self(value)
    }
}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, PartialEq)]
pub enum Type<'src> {
    Unit,
    Int,
    Forall {
        param_name: Spanned<&'src str>,
        body_type: Box<Self>,
    },
    Fun {
        param_type: Box<Self>,
        body_type: Box<Self>,
    },
    Var {
        name: Spanned<&'src str>,
    },
}

impl Display for Type<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Int => write!(f, "Int",),
            Self::Forall {
                param_name,
                body_type,
            } => write!(f, "∀{}. {}", param_name.inner, body_type),
            Self::Fun {
                param_type,
                body_type,
            } => match param_type.as_ref() {
                Self::Fun { .. } | Self::Forall { .. } => write!(f, "({param_type}) → {body_type}"),
                _ => write!(f, "{param_type} → {body_type}"),
            },
            Self::Var { name } => write!(f, "{}", name.inner),
        }
    }
}

pub enum Term<'src> {
    Unit,
    Int {
        value: usize,
    },
    Lam {
        param_name: Spanned<&'src str>,
        param_type: Type<'src>,
        body: Box<Self>,
    },
    TypeLam {
        param_name: Spanned<&'src str>,
        body: Box<Self>,
    },
    Var {
        name: Spanned<&'src str>,
    },
    App {
        callee: Spanned<Box<Self>>,
        arg: Spanned<Box<Self>>,
    },
    TypeApp {
        callee: Spanned<Box<Self>>,
        arg: Type<'src>,
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
            Self::TypeLam { param_name, body } => write!(f, "Λ{}. {}", param_name.inner, body),
            Self::App { callee, arg } => match callee.as_ref() {
                Self::Lam { .. } | Self::TypeLam { .. } => write!(f, "({callee}) {arg}"),
                _ => write!(f, "{callee} {arg}"),
            },
            Self::TypeApp { callee, arg } => match callee.as_ref() {
                Self::Lam { .. } | Self::TypeLam { .. } => write!(f, "({callee}) {arg}"),
                _ => write!(f, "{callee} {arg}"),
            },
        }
    }
}

fn parens<'src, T>(
    parser: impl Parser<'src, &'src str, T, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, T, Err<Error<'src>>> + Clone {
    parser.delimited_by(just('(').padded(), just(')'))
}

fn unit_type<'src>() -> impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone {
    just("()").map(|_| Type::Unit)
}

fn int_type<'src>() -> impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone {
    keyword("Int").map(|_| Type::Int)
}

fn forall_type<'src>(
    ty: impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone {
    keyword("forall")
        .padded()
        .ignore_then(name().spanned().map(Into::into))
        .padded()
        .then_ignore(just('.'))
        .padded()
        .then(ty.map(Box::new))
        .map(|(param_name, body_type)| Type::Forall {
            param_name,
            body_type,
        })
}

fn fun_type<'src>(
    ty: impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone {
    ty.clone().padded().foldl(
        just("->").padded().ignore_then(ty).padded().repeated(),
        |param, body| Type::Fun {
            param_type: Box::new(param),
            body_type: Box::new(body),
        },
    )
}

fn var_type<'src>() -> impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone {
    name().spanned().map(|name| Type::Var { name: name.into() })
}

fn ty<'src>() -> impl Parser<'src, &'src str, Type<'src>, Err<Error<'src>>> + Clone {
    let mut ty = Recursive::declare();
    ty.define({
        let ty = choice((
            unit_type(),
            int_type(),
            forall_type(ty.clone()),
            var_type(),
            parens(ty.clone()),
        ));
        fun_type(ty)
    });
    ty
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
        .ignore_then(name().spanned().map(Into::into))
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

fn type_lam_term<'src>(
    term: impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("Lam")
        .padded()
        .ignore_then(name().spanned().map(Into::into))
        .padded()
        .then_ignore(just('.'))
        .padded()
        .then(term.map(Box::new))
        .map(|(param_name, body)| Term::TypeLam { param_name, body })
}

fn var_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    name().spanned().map(|name| Term::Var { name: name.into() })
}

fn app_term<'src>(
    term: impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    enum Arg<'src> {
        TermArg(Term<'src>),
        TypeArg(Type<'src>),
    }
    let arg = choice((
        ty().delimited_by(just('[').padded(), just(']'))
            .map(Arg::TypeArg),
        term.clone().map(Arg::TermArg),
    ));
    term.spanned()
        .padded()
        .foldl(arg.spanned().padded().repeated(), |callee, arg| {
            let span: SimpleSpan = (callee.span.start..arg.span.end).into();
            match arg.inner {
                Arg::TermArg(term) => Term::App {
                    callee: Box::new(callee.inner).with_span(callee.span).into(),
                    arg: Box::new(term).with_span(arg.span).into(),
                },
                Arg::TypeArg(ty) => Term::TypeApp {
                    callee: Box::new(callee.inner).with_span(callee.span).into(),
                    arg: ty,
                },
            }
            .with_span(span)
        })
        .map(|spanned| spanned.inner)
}

pub fn term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> {
    let mut term = Recursive::declare();
    term.define({
        let term = choice((
            unit_term(),
            int_term(),
            lam_term(term.clone()),
            type_lam_term(term.clone()),
            var_term(),
            parens(term.clone()),
        ));
        app_term(term)
    });
    term
}
