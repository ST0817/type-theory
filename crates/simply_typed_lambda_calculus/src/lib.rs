use chumsky::Parser;
use parsers::Result;

pub mod check;
pub mod parser;

pub fn run<'src>(src: &'src str) -> Result<'src, ()> {
    let term = parser::term().parse(src).into_result()?;
    println!("{term}");
    let raw_type = check::check_term(&term);
    println!("=> {raw_type}");
    Ok(())
}
