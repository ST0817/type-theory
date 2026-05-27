use std::collections::HashMap;

use chumsky::span::SimpleSpan;
use parsers::{Error, Result};

use crate::parser::{Term, Type};

pub type Context = HashMap<String, Type>;

fn check_type_match<'src>(type1: &Type, type2: &Type, span: &SimpleSpan) -> Result<'src, ()> {
    if type1 != type2 {
        return Err(vec![Error::custom(
            *span,
            format!("type mismatch: {type1} and {type2}"),
        )]);
    }
    Ok(())
}

pub fn check_term<'src>(term: &Term<'src>, context: &Context) -> Result<'src, Type> {
    match term {
        Term::Unit => Ok(Type::Unit),
        Term::Int { value: _ } => Ok(Type::Int),
        Term::Lam {
            param_name,
            param_type,
            body,
        } => {
            let mut new_context = context.clone();
            new_context.insert(param_name.to_string(), param_type.clone());
            let body_type = check_term(body, &new_context)?;
            Ok(Type::Fun {
                param_type: Box::new(param_type.clone()),
                body_type: Box::new(body_type),
            })
        }
        Term::Var { name } => context
            .get(name.inner)
            .cloned()
            .ok_or_else(|| vec![Error::custom(name.span, "unbound variable")]),
        Term::App { callee, arg } => {
            let Type::Fun {
                param_type,
                body_type,
            } = check_term(callee, context)?
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let arg_type = check_term(arg, context)?;
            check_type_match(&param_type, &arg_type, &arg.span)?;
            Ok(*body_type)
        }
    }
}
