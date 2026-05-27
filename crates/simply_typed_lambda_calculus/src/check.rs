use std::collections::HashMap;

use parsers::{Error, Result};

use crate::parser::{Term, Type};

pub type Context = HashMap<String, Type>;

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
                param: Box::new(param_type.clone()),
                body: Box::new(body_type),
            })
        }
        Term::Var { name } => context
            .get(name.inner)
            .cloned()
            .ok_or_else(|| vec![Error::custom(name.span, "unbound variable")]),
    }
}
