use chumsky::Parser;
use parsers::Result;

use crate::check::{Context, TypeContext};

pub mod check;
pub mod parser;

pub fn run<'src>(src: &'src str) -> Result<'src, ()> {
    let term = parser::term().parse(src).into_result()?;
    println!("{term}");
    let context = Context::new();
    let type_context = TypeContext::new();
    let ty = check::check_term(&term, &context, &type_context)?;
    println!("=> {ty}");
    Ok(())
}
