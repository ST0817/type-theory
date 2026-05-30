use std::collections::HashMap;

use chumsky::span::SimpleSpan;
use parsers::{Error, Name, Result};

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
        /*
         *
         * ───────────
         * Γ ⊢ () : ()
         */
        Term::Unit => Ok(Type::Unit),

        /*
         *
         * ───────────────────
         * Γ ⊢ <integer> : Int
         */
        Term::Int { value: _ } => Ok(Type::Int),

        /*
         *      Γ, x : σ ⊢ e : τ
         * ────────────────────────────
         * Γ ⊢ (lam x : σ. e) : (σ → τ)
         */
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

        /*
         * x : σ ∈ Γ
         * ─────────
         * Γ ⊢ x : σ
         */
        Term::Var { name } => context
            .get(name.inner)
            .cloned()
            .ok_or_else(|| vec![Error::custom(name.span, "unbound variable")]),

        /*
         * Γ, e₁ : σ → τ  Γ ⊢ e₂ : σ
         * ─────────────────────────
         *   Γ ⊢ e₁ e₂ : (σ → τ)
         */
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

pub fn check_def<'src>(
    name: Name<'src>,
    term: &Term<'src>,
    context: &mut Context,
) -> Result<'src, ()> {
    let ty = check_term(term, context)?;
    context.insert(name.to_string(), ty);
    Ok(())
}
