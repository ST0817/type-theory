use std::{cmp::max, collections::HashMap};

use chumsky::span::{SimpleSpan, SpanWrap, Spanned};
use parsers::{Error, Result};

use crate::parser::Term;

pub type Context = HashMap<String, Term>;

fn check_sort<'src>(term: &Term, span: &SimpleSpan) -> Result<'src, usize> {
    let Term::Sort { level } = term else {
        return Err(vec![Error::custom(*span, "not a sort")]);
    };
    Ok(*level)
}

fn check_type_match<'src>(type1: &Term, type2: &Term, span: &SimpleSpan) -> Result<'src, ()> {
    if type1 != type2 {
        return Err(vec![Error::custom(
            *span,
            format!("type mismatch: {type1} and {type2}"),
        )]);
    }
    Ok(())
}

pub fn check_term<'src>(term: &Term, context: &Context) -> Result<'src, Term> {
    match term {
        /*
         *
         * ─────────────────────────
         * Γ ⊢ Sort n : Sort (n + 1)
         */
        Term::Sort { level } => Ok(Term::Sort { level: level + 1 }),

        /*
         *
         * ─────────────
         * Γ ⊢ () : Unit
         */
        Term::Unit => Ok(Term::UnitType),

        /*
         *
         * ─────────────────
         * Γ ⊢ Unit : Sort 1
         */
        Term::UnitType => Ok(Term::Sort { level: 1 }),

        /*
         *
         * ───────────────────
         * Γ ⊢ <integer> : Int
         */
        Term::Int { .. } => Ok(Term::IntType),

        /*
         *
         * ────────────────
         * Γ ⊢ Int : Sort 1
         */
        Term::IntType => Ok(Term::Sort { level: 1 }),

        /*
         *      Γ, x : α ⊢ t : β
         * ───────────────────────────
         * Γ ⊢ (λx : α. t) : Πx : α. β
         */
        Term::Lam {
            param_name,
            param_type,
            body,
        } => {
            let checked_param_type = check_term(param_type, context)?;
            check_sort(&checked_param_type, &param_type.span)?;
            let mut new_context = context.clone();
            new_context.insert(param_name.to_string(), param_type.as_ref().clone());
            let body_type = check_term(body, &new_context)?;
            let ty = Term::Pi {
                param_name: param_name.to_string(),
                param_type: param_type.clone(),
                body_type: Box::new(body_type).with_span(SimpleSpan::default()),
            };
            Ok(ty)
        }

        /*
         * Γ ⊢ α : Sort m, Γ, x : α ⊢ β : Sort n
         * ─────────────────────────────────────
         *   Γ ⊢ Πx : α. β : Sort (max m n)
         */
        Term::Pi {
            param_name,
            param_type,
            body_type,
        } => {
            let checled_param_type = check_term(param_type, context)?;
            let param_sort_level = check_sort(&checled_param_type, &param_type.span)?;
            let mut new_context = context.clone();
            new_context.insert(param_name.to_string(), param_type.as_ref().clone());
            let checked_body_type = check_term(body_type, &new_context)?;
            let body_sort_level = check_sort(&checked_body_type, &body_type.span)?;
            Ok(Term::Sort {
                level: max(param_sort_level, body_sort_level),
            })
        }

        /*
         * x : α ∈ Γ
         * ─────────
         * Γ ⊢ x : α
         */
        Term::Var { name } => context
            .get(&name.inner)
            .cloned()
            .ok_or_else(|| vec![Error::custom(name.span, "unbound variable")]),

        /*
         * Γ, e₁ : α → β  Γ ⊢ e₂ : α
         * ─────────────────────────
         *   Γ ⊢ e₁ e₂ : β
         */
        Term::App { callee, arg } => {
            let Term::Pi {
                param_type,
                body_type,
                ..
            } = check_term(callee, context)?
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let arg_type = check_term(arg, context)?;
            check_type_match(&param_type, &arg_type, &arg.span)?;
            Ok(body_type.as_ref().clone())
        }
    }
}

pub fn check_def<'src>(
    name: Spanned<&'src str>,
    term: &Term,
    context: &mut Context,
) -> Result<'src, ()> {
    let ty = check_term(term, context)?;
    context.insert(name.to_string(), ty);
    Ok(())
}
