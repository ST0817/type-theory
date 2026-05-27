use chumsky::{Parser, error::Rich, extra::Err, prelude::any, span::Spanned, text};

pub type Error<'src> = Rich<'src, char>;
pub type Result<'src, T> = std::result::Result<T, Vec<Error<'src>>>;

pub type Name<'src> = Spanned<&'src str>;

pub fn int<'src>() -> impl Parser<'src, &'src str, usize, Err<Error<'src>>> + Clone {
    text::int(10).from_str().unwrapped()
}

pub fn name<'src>() -> impl Parser<'src, &'src str, Name<'src>, Err<Error<'src>>> + Clone {
    any()
        .filter(char::is_ascii_alphabetic)
        .repeated()
        .to_slice()
        .spanned()
}
