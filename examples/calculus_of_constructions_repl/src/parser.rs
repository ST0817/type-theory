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
        name: Spanned<&'src str>,
        term: Term,
    },
    Term {
        term: Term,
    },
}

fn def_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    just(':')
        .padded()
        .ignore_then(keyword("def"))
        .padded()
        .ignore_then(name().spanned().map(Into::into))
        .padded()
        .then_ignore(just(":="))
        .padded()
        .then(term())
        .map(|(name, term)| ReplCmd::Def { name, term })
}

fn term_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    term().map(|term| ReplCmd::Term { term })
}

pub fn repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    choice((def_repl_cmd(), term_repl_cmd()))
}
