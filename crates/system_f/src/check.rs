use core::fmt;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use chumsky::span::SimpleSpan;
use parsers::{Error, Name, Result};

use crate::parser::{Term, Type};

#[derive(Clone)]
pub enum RawType {
    Unit,
    Int,
    Var {
        name: String,
        index: usize,
    },
    Forall {
        param_name: String,
        body_type: Box<Self>,
    },
    Fun {
        param_type: Box<Self>,
        body_type: Box<Self>,
    },
}

impl PartialEq for RawType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) | (Self::Int, Self::Int) => true,
            (Self::Var { index: index1, .. }, Self::Var { index: index2, .. }) => index1 == index2,
            (
                Self::Forall {
                    body_type: body_type1,
                    ..
                },
                Self::Forall {
                    body_type: body_type2,
                    ..
                },
            ) => body_type1 == body_type2,
            (
                Self::Fun {
                    param_type: param_type1,
                    body_type: body_type1,
                },
                Self::Fun {
                    param_type: param_type2,
                    body_type: body_type2,
                },
            ) => param_type1 == param_type2 && body_type1 == body_type2,
            _ => false,
        }
    }
}

impl<'src> Display for RawType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "()"),
            Self::Int => write!(f, "Int",),
            Self::Forall {
                param_name,
                body_type,
            } => write!(f, "∀{param_name}. {body_type}"),
            Self::Fun {
                param_type,
                body_type,
            } => match param_type.as_ref() {
                Self::Fun { .. } | Self::Forall { .. } => write!(f, "({param_type}) → {body_type}"),
                _ => write!(f, "{param_type} → {body_type}"),
            },
            Self::Var { name, index } => write!(f, "{name}({index})"),
        }
    }
}

impl<'src> RawType {
    fn subst(&self, ty_name: &str, ty: &RawType) -> Self {
        match self {
            Self::Var { name, index } => {
                if *name == ty_name {
                    ty.clone()
                } else {
                    Self::Var {
                        name: name.clone(),
                        index: *index - 1,
                    }
                }
            }
            Self::Forall {
                param_name,
                body_type,
            } => Self::Forall {
                param_name: param_name.clone(),
                body_type: Box::new(body_type.subst(ty_name, ty)),
            },
            Self::Fun {
                param_type,
                body_type,
            } => Self::Fun {
                param_type: Box::new(param_type.subst(ty_name, ty)),
                body_type: Box::new(body_type.subst(ty_name, ty)),
            },
            _ => ty.clone(),
        }
    }
}

pub type Context = HashMap<String, RawType>;
pub type TypeContext<'src> = Vec<String>;

fn check_type_match<'src>(type1: &RawType, type2: &RawType, span: &SimpleSpan) -> Result<'src, ()> {
    if type1 != type2 {
        return Err(vec![Error::custom(
            *span,
            format!("type mismatch: {type1} and {type2}"),
        )]);
    }
    Ok(())
}

fn check_type<'src>(ty: &Type<'src>, type_context: &TypeContext<'src>) -> Result<'src, RawType> {
    match ty {
        Type::Unit => Ok(RawType::Unit),
        Type::Int => Ok(RawType::Int),
        Type::Var { name } => type_context
            .iter()
            .rev()
            .position(|type_name| type_name == name.inner)
            .map(|index| RawType::Var {
                name: name.to_string(),
                index,
            })
            .ok_or_else(|| vec![Error::custom(name.span, "unbound type")]),
        Type::Forall {
            param_name,
            body_type,
        } => {
            let mut new_type_context = type_context.clone();
            new_type_context.push(param_name.to_string());
            let body_type = check_type(body_type, &new_type_context)?;
            Ok(RawType::Forall {
                param_name: param_name.to_string(),
                body_type: Box::new(body_type),
            })
        }
        Type::Fun {
            param_type,
            body_type,
        } => Ok(RawType::Fun {
            param_type: Box::new(check_type(param_type, type_context)?),
            body_type: Box::new(check_type(body_type, type_context)?),
        }),
    }
}

pub fn check_term<'src>(
    term: &Term<'src>,
    context: &Context,
    type_context: &TypeContext<'src>,
) -> Result<'src, RawType> {
    match term {
        /*
         *
         * ───────────
         * Γ ⊢ () : ()
         */
        Term::Unit => Ok(RawType::Unit),

        /*
         *
         * ───────────────────
         * Γ ⊢ <integer> : Int
         */
        Term::Int { value: _ } => Ok(RawType::Int),

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
            let raw_param_type = check_type(param_type, type_context)?;
            let mut new_context = context.clone();
            new_context.insert(param_name.to_string(), raw_param_type.clone());
            let body_type = check_term(body, &new_context, type_context)?;
            Ok(RawType::Fun {
                param_type: Box::new(raw_param_type),
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
            new_type_context.push(param_name.to_string());
            let raw_body_type = check_term(body, &context, &new_type_context)?;
            Ok(RawType::Forall {
                param_name: param_name.to_string(),
                body_type: Box::new(raw_body_type),
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
            let RawType::Fun {
                param_type,
                body_type,
            } = check_term(callee, context, type_context)?
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let arg_type = check_term(arg, context, type_context)?;
            check_type_match(&arg_type, &param_type, &arg.span)?;
            Ok(*body_type)
        }

        /*
         *   Γ ⊢ M : ∀α. σ
         * ──────────────────
         * Γ ⊢ M [τ] : σ[τ/α]
         */
        Term::TypeApp { callee, arg } => {
            let RawType::Forall {
                param_name,
                body_type,
            } = check_term(callee, context, type_context)?
            else {
                return Err(vec![Error::custom(callee.span, "not a type function")]);
            };
            let raw_arg_type = check_type(arg, type_context)?;
            Ok(body_type.subst(&param_name, &raw_arg_type))
        }
    }
}

pub fn check_def<'src>(
    name: Name<'src>,
    term: &Term<'src>,
    context: &mut Context,
    type_context: &TypeContext<'src>,
) -> Result<'src, ()> {
    let ty = check_term(term, context, type_context)?;
    context.insert(name.to_string(), ty);
    Ok(())
}
