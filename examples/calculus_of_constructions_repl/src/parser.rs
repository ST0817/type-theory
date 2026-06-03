use calculus_of_constructions::parser::{Term, term};
use chumsky::{
    Parser,
    extra::Err,
    prelude::{choice, just},
    span::Spanned,
    text::ascii::keyword,
};
use parsers::{Error, name};

pub enum ReplCmd<'src> {
    Def {
        name: &'src str,
        term: Term<'src>,
    },
    Axiom {
        name: &'src str,
        term: Spanned<Term<'src>>,
    },
    Term {
        term: Term<'src>,
    },
}

fn def_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    just(':')
        .padded()
        .ignore_then(keyword("def"))
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(":="))
        .padded()
        .then(term())
        .map(|(name, term)| ReplCmd::Def { name, term })
}

fn axiom_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    just(':')
        .padded()
        .ignore_then(keyword("axiom"))
        .padded()
        .ignore_then(name())
        .padded()
        .then_ignore(just(':'))
        .padded()
        .then(term().spanned())
        .map(|(name, term)| ReplCmd::Axiom { name, term })
}

fn term_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    term().map(|term| ReplCmd::Term { term })
}

pub fn repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    choice((def_repl_cmd(), axiom_repl_cmd(), term_repl_cmd()))
}
