use chumsky::{
    Parser,
    extra::Err,
    prelude::{choice, just},
    text::ascii::keyword,
};
use parsers::{Error, int};

use crate::syntax::ExprSyntax;

fn unit_expr_syntax<'src>() -> impl Parser<'src, &'src str, ExprSyntax, Err<Error<'src>>> + Clone {
    just("()").map(|_| ExprSyntax::Unit)
}

fn unit_type_expr_syntax<'src>()
-> impl Parser<'src, &'src str, ExprSyntax, Err<Error<'src>>> + Clone {
    keyword("Unit").map(|_| ExprSyntax::UnitType)
}

fn nat_expr_syntax<'src>() -> impl Parser<'src, &'src str, ExprSyntax, Err<Error<'src>>> + Clone {
    int().map(|value| ExprSyntax::Nat { value })
}

fn nat_type_expr_syntax<'src>() -> impl Parser<'src, &'src str, ExprSyntax, Err<Error<'src>>> + Clone
{
    keyword("Nat").map(|_| ExprSyntax::NatType)
}

fn sort_expr_syntax<'src>() -> impl Parser<'src, &'src str, ExprSyntax, Err<Error<'src>>> + Clone {
    keyword("Sort")
        .padded()
        .ignore_then(int())
        .map(|level| ExprSyntax::Sort { level })
}

pub fn expr_syntax<'src>() -> impl Parser<'src, &'src str, ExprSyntax, Err<Error<'src>>> + Clone {
    choice((
        unit_expr_syntax(),
        unit_type_expr_syntax(),
        nat_expr_syntax(),
        nat_type_expr_syntax(),
        sort_expr_syntax(),
    ))
}
