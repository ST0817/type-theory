use chumsky::{Parser, error::Rich, extra::Err, prelude::any, text};

pub type Error<'src> = Rich<'src, char>;
pub type Result<'src, T> = std::result::Result<T, Vec<Error<'src>>>;

pub fn int<'src>() -> impl Parser<'src, &'src str, usize, Err<Error<'src>>> + Clone {
    text::int(10).from_str().unwrapped()
}

pub fn name<'src>() -> impl Parser<'src, &'src str, &'src str, Err<Error<'src>>> + Clone {
    any()
        .filter(char::is_ascii_alphabetic)
        .repeated()
        .at_least(1)
        .to_slice()
}
