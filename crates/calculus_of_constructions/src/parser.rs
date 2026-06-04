use chumsky::{
    IterParser, Parser,
    extra::Err,
    prelude::{Recursive, choice, just},
    span::{SimpleSpan, SpanWrap, Spanned},
    text::ascii::keyword,
};
use parsers::{Error, int, name};

#[derive(Clone, PartialEq)]
pub enum Term<'src> {
    Sort {
        level: usize,
    },
    Unit,
    UnitType,
    Int {
        value: usize,
    },
    IntType,
    Lam {
        param_name: &'src str,
        param_type: Spanned<Box<Self>>,
        body: Box<Self>,
    },
    Pi {
        param_name: &'src str,
        param_type: Spanned<Box<Self>>,
        body_type: Spanned<Box<Self>>,
    },
    Var {
        name: Spanned<&'src str>,
    },
    App {
        callee: Spanned<Box<Self>>,
        arg: Spanned<Box<Self>>,
    },
}

fn sort_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("Sort")
        .padded()
        .ignore_then(int())
        .map(|level| Term::Sort { level })
}

fn prop_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("Prop").map(|_| Term::Sort { level: 0 })
}

fn type_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("Type")
        .padded()
        .ignore_then(int().or_not())
        .map(|level| Term::Sort {
            level: level.unwrap_or(0) + 1,
        })
}

fn unit_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    just("()").map(|_| Term::Unit)
}

fn unit_type_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("Unit").map(|_| Term::UnitType)
}

fn int_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    int().map(|value| Term::Int { value })
}

fn int_type_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("Int").map(|_| Term::IntType)
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
        .then(term.clone().map(Box::new).spanned())
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

fn pi_term_with_name<'src>(
    term: impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    keyword("pi")
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(':'))
        .padded()
        .then(term.clone().map(Box::new).spanned())
        .padded()
        .then_ignore(just('.'))
        .padded()
        .then(term.map(Box::new).spanned())
        .map(|((param_name, param_type), body_type)| Term::Pi {
            param_name,
            param_type,
            body_type,
        })
}

fn var_term<'src>() -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    name().spanned().map(|name| Term::Var { name })
}

fn pi_term_no_name<'src>(
    term: impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone,
) -> impl Parser<'src, &'src str, Term<'src>, Err<Error<'src>>> + Clone {
    term.clone()
        .spanned()
        .padded()
        .then_ignore(just("->"))
        .padded()
        .repeated()
        .foldr(term.spanned(), |param_type, body_type| {
            let span: SimpleSpan = (param_type.span.start..body_type.span.end).into();
            Term::Pi {
                param_name: "",
                param_type: Box::new(param_type.inner).with_span(param_type.span),
                body_type: Box::new(body_type.inner).with_span(body_type.span),
            }
            .with_span(span)
        })
        .map(|spanned| spanned.inner)
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
        let term = choice((
            sort_term(),
            prop_term(),
            type_term(),
            unit_term(),
            unit_type_term(),
            int_term(),
            int_type_term(),
            lam_term(term.clone()),
            pi_term_with_name(term.clone()),
            var_term(),
            term.clone().delimited_by(just('(').padded(), just(')')),
        ));
        let term = app_term(term);
        pi_term_no_name(term)
    });
    term
}
