use std::collections::HashMap;

use chumsky::span::SimpleSpan;
use parsers::{Error, Result};

use crate::parser::{Term, Type};

pub type Context<'src> = HashMap<&'src str, Type<'src>>;
pub type TypeContext<'src> = Vec<&'src str>;

fn check_type_match<'src>(
    type1: &Type<'src>,
    type2: &Type<'src>,
    span: &SimpleSpan,
) -> Result<'src, ()> {
    if type1 != type2 {
        return Err(vec![Error::custom(
            *span,
            format!("type mismatch: {type1} and {type2}"),
        )]);
    }
    Ok(())
}

impl<'src> Type<'src> {
    fn subst(&self, type_name: &str, ty: &Type<'src>) -> Type<'src> {
        match self {
            Self::Var { name } if name.inner == type_name => ty.clone(),
            Self::Forall {
                param_name,
                body_type,
            } if param_name.inner != type_name => Self::Forall {
                param_name: *param_name,
                body_type: Box::new(body_type.subst(type_name, ty)),
            },
            Self::Fun {
                param_type,
                body_type,
            } => Self::Fun {
                param_type: Box::new(param_type.subst(type_name, ty)),
                body_type: Box::new(body_type.subst(type_name, ty)),
            },
            _ => self.clone(),
        }
    }
}

fn check_type<'src>(ty: &Type<'src>, type_context: &TypeContext<'src>) -> Result<'src, ()> {
    match ty {
        Type::Var { name } if !type_context.contains(&name.inner) => {
            Err(vec![Error::custom(name.span, "undefined type name")])
        }
        Type::Forall {
            param_name,
            body_type,
        } => {
            let mut new_type_context = type_context.clone();
            new_type_context.push(param_name.inner);
            check_type(body_type, &new_type_context)
        }
        Type::Fun {
            param_type,
            body_type,
        } => {
            check_type(param_type, type_context)?;
            check_type(body_type, type_context)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn check_term<'src>(
    term: &Term<'src>,
    context: &Context<'src>,
    type_context: &TypeContext<'src>,
) -> Result<'src, Type<'src>> {
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
            check_type(param_type, type_context)?;
            let mut new_context = context.clone();
            new_context.insert(param_name, param_type.clone());
            let body_type = check_term(body, &new_context, type_context)?;
            Ok(Type::Fun {
                param_type: Box::new(param_type.clone()),
                body_type: Box::new(body_type),
            })
        }

        /*
         *   Γ, α type ⊢ M : σ
         * ──────────────────────
         * Γ ⊢ (Lam α. M) : ∀α. σ
         */
        Term::TypeLam { param_name, body } => {
            let mut new_type_context = type_context.clone();
            new_type_context.push(param_name);
            let body_type = check_term(body, &context, &new_type_context)?;
            Ok(Type::Forall {
                param_name: *param_name,
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
            } = check_term(callee, context, type_context)?
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let arg_type = check_term(arg, context, type_context)?;
            check_type_match(&param_type, &arg_type, &arg.span)?;
            Ok(*body_type)
        }

        /*
         *   Γ ⊢ M : ∀α. σ
         * ──────────────────
         * Γ ⊢ M [τ] : σ[τ/α]
         */
        Term::TypeApp { callee, arg } => {
            let Type::Forall {
                param_name,
                body_type,
            } = check_term(callee, context, type_context)?
            else {
                return Err(vec![Error::custom(callee.span, "not a type function")]);
            };
            check_type(arg, type_context)?;
            Ok(body_type.subst(param_name.inner, arg))
        }
    }
}
