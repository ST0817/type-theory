use chumsky::Parser;
use parsers::Result;

use crate::check::Context;

pub mod check;
pub mod parser;

pub fn run<'src>(src: &'src str) -> Result<'src, ()> {
    let term = parser::term().parse(src).into_result()?;
    println!("{term}");
    let context = Context::new();
    let ty = check::check_term(&term, &context)?;
    println!("=> {ty}");
    Ok(())
}
