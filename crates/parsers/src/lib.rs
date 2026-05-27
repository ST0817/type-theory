use chumsky::{Parser, error::Rich, extra::Err, text};

pub type Error<'src> = Rich<'src, char>;
pub type Result<'src, T> = std::result::Result<T, Vec<Error<'src>>>;

pub fn int<'src>() -> impl Parser<'src, &'src str, usize, Err<Error<'src>>> + Clone {
    text::int(10).from_str().unwrapped()
}
