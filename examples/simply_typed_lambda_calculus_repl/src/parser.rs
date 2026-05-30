use chumsky::{
    Parser,
    extra::Err,
    prelude::{choice, just},
    text::ascii::keyword,
};
use parsers::{Error, Name, name};
use simply_typed_lambda_calculus::parser::{Term, term};

pub enum ReplCmd<'src> {
    Def { name: Name<'src>, term: Term<'src> },
    Term { term: Term<'src> },
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

fn term_repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    term().map(|term| ReplCmd::Term { term })
}

pub fn repl_cmd<'src>() -> impl Parser<'src, &'src str, ReplCmd<'src>, Err<Error<'src>>> {
    choice((def_repl_cmd(), term_repl_cmd()))
}
