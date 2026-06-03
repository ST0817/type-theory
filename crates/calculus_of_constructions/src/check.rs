use std::{
    cmp::max,
    fmt::{self, Display, Formatter},
};

use chumsky::span::{SimpleSpan, Spanned};
use ignorable::PartialEq;
use indexmap::IndexMap;
use parsers::{Error, Result};

use crate::parser::Term;

#[derive(Clone, Debug, PartialEq)]
pub enum CheckedTerm {
    Sort {
        level: usize,
    },
    Unit,
    UnitType,
    Int {
        value: usize,
    },
    IntType,
    Lam {
        #[ignored(PartialEq)]
        param_name: String,
        param_type: Box<Self>,
        body: Box<Self>,
    },
    Pi {
        #[ignored(PartialEq)]
        param_name: String,
        param_type: Box<Self>,
        body_type: Box<Self>,
    },
    Var {
        #[ignored(PartialEq)]
        name: String,
        index: usize,
    },
    App {
        callee: Box<Self>,
        arg: Box<Self>,
    },
}

#[test]
fn test_alpha_eq() {
    assert_eq!(
        CheckedTerm::Lam {
            param_name: "x".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Var {
                name: "x".to_string(),
                index: 0
            })
        },
        CheckedTerm::Lam {
            param_name: "y".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Var {
                name: "y".to_string(),
                index: 0
            })
        }
    );
}

impl Display for CheckedTerm {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Sort { level } => write!(f, "Sort {level}"),
            Self::Unit => write!(f, "()"),
            Self::UnitType => write!(f, "Unit"),
            Self::Int { value } => write!(f, "{value}"),
            Self::IntType => write!(f, "Int"),
            Self::Var { name, index } => write!(f, "{name}#{index}"),
            Self::Lam {
                param_name,
                param_type,
                body,
            } => write!(f, "λ{param_name} : {param_type}. {body}"),
            Self::Pi {
                param_name,
                param_type,
                body_type,
            } => write!(f, "Π{param_name} : {param_type}. {body_type}",),
            Self::App { callee, arg } => {
                match **callee {
                    Self::Lam { .. } => write!(f, "({callee}) ")?,
                    _ => write!(f, "{callee} ")?,
                }
                match **arg {
                    Self::App { .. } => write!(f, "({arg})")?,
                    _ => write!(f, "{arg}")?,
                }
                Ok(())
            }
        }
    }
}

impl CheckedTerm {
    fn shift(&self, value: isize, cutoff: usize) -> Self {
        match self {
            Self::Var { name, index } if *index >= cutoff => Self::Var {
                name: name.clone(),
                index: (*index as isize + value) as usize,
            },
            Self::Lam {
                param_name,
                param_type,
                body,
            } => Self::Lam {
                param_name: param_name.clone(),
                param_type: Box::new(param_type.shift(value, cutoff)),
                body: Box::new(body.shift(value, cutoff + 1)),
            },
            Self::Pi {
                param_name,
                param_type,
                body_type,
            } => Self::Pi {
                param_name: param_name.clone(),
                param_type: Box::new(param_type.shift(value, cutoff)),
                body_type: Box::new(body_type.shift(value, cutoff + 1)),
            },
            Self::App { callee, arg } => Self::App {
                callee: Box::new(callee.shift(value, cutoff)),
                arg: Box::new(arg.shift(value, cutoff)),
            },
            _ => self.clone(),
        }
    }

    fn subst_at(&self, term: &CheckedTerm, depth: usize) -> Self {
        match self {
            Self::Var { index, .. } if *index == depth => term.shift(depth as isize, 0),
            Self::Var { name, index } if *index > depth => Self::Var {
                name: name.clone(),
                index: index - 1,
            },
            Self::Lam {
                param_name,
                param_type,
                body,
            } => Self::Lam {
                param_name: param_name.clone(),
                param_type: Box::new(param_type.subst_at(term, depth)),
                body: Box::new(body.subst_at(term, depth + 1)),
            },
            Self::Pi {
                param_name,
                param_type,
                body_type,
            } => Self::Pi {
                param_name: param_name.clone(),
                param_type: Box::new(param_type.subst_at(term, depth)),
                body_type: Box::new(body_type.subst_at(term, depth + 1)),
            },
            Self::App { callee, arg } => Self::App {
                callee: Box::new(callee.subst_at(term, depth)),
                arg: Box::new(arg.subst_at(term, depth)),
            },
            _ => self.clone(),
        }
    }

    fn subst(&self, term: &CheckedTerm) -> Self {
        self.subst_at(term, 0)
    }

    fn whnf(&self) -> Self {
        match self {
            Self::App { callee, arg } => match callee.whnf() {
                Self::Lam { body, .. } => body.subst(arg).whnf(),
                other => Self::App {
                    callee: Box::new(other),
                    arg: arg.clone(),
                },
            },
            _ => self.clone(),
        }
    }

    fn normalize(&self) -> Self {
        match self {
            Self::Lam {
                param_name,
                param_type,
                body,
            } => Self::Lam {
                param_name: param_name.clone(),
                param_type: Box::new(param_type.normalize()),
                body: Box::new(body.normalize()),
            },
            Self::Pi {
                param_name,
                param_type,
                body_type,
            } => Self::Pi {
                param_name: param_name.clone(),
                param_type: Box::new(param_type.normalize()),
                body_type: Box::new(body_type.normalize()),
            },
            Self::App { callee, arg } => match callee.normalize() {
                Self::Lam { body, .. } => body.subst(&arg.normalize()).normalize(),
                other => Self::App {
                    callee: Box::new(other),
                    arg: Box::new(arg.normalize()),
                },
            },
            _ => self.clone(),
        }
    }
}

fn check_beta_eq(term1: &CheckedTerm, term2: &CheckedTerm) -> bool {
    term1.normalize() == term2.normalize()
}

#[test]
fn test_whnf() {
    assert_eq!(
        // (lam x : Int. lam y : Int. (lam z : Int. z) 53 ) 42
        CheckedTerm::App {
            callee: Box::new(CheckedTerm::Lam {
                param_name: "x".to_string(),
                param_type: Box::new(CheckedTerm::IntType),
                body: Box::new(CheckedTerm::Lam {
                    param_name: "y".to_string(),
                    param_type: Box::new(CheckedTerm::IntType),
                    body: Box::new(CheckedTerm::App {
                        callee: Box::new(CheckedTerm::Lam {
                            param_name: "z".to_string(),
                            param_type: Box::new(CheckedTerm::IntType),
                            body: Box::new(CheckedTerm::Var {
                                name: "z".to_string(),
                                index: 0
                            }),
                        }),
                        arg: Box::new(CheckedTerm::Int { value: 53 })
                    })
                }),
            }),
            arg: Box::new(CheckedTerm::Int { value: 42 }),
        }
        .whnf(),
        // lam y : Int. (lam z : Int. z) 53
        CheckedTerm::Lam {
            param_name: "y".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::App {
                callee: Box::new(CheckedTerm::Lam {
                    param_name: "z".to_string(),
                    param_type: Box::new(CheckedTerm::IntType),
                    body: Box::new(CheckedTerm::Var {
                        name: "z".to_string(),
                        index: 0
                    }),
                }),
                arg: Box::new(CheckedTerm::Int { value: 53 })
            })
        },
    )
}

#[test]
fn test_beta_eq() {
    assert!(check_beta_eq(
        // (lam x : Int. lam y : Int. (lam z : Int. z) 53 ) 42
        &CheckedTerm::App {
            callee: Box::new(CheckedTerm::Lam {
                param_name: "x".to_string(),
                param_type: Box::new(CheckedTerm::IntType),
                body: Box::new(CheckedTerm::Lam {
                    param_name: "y".to_string(),
                    param_type: Box::new(CheckedTerm::IntType),
                    body: Box::new(CheckedTerm::App {
                        callee: Box::new(CheckedTerm::Lam {
                            param_name: "z".to_string(),
                            param_type: Box::new(CheckedTerm::IntType),
                            body: Box::new(CheckedTerm::Var {
                                name: "z".to_string(),
                                index: 0
                            }),
                        }),
                        arg: Box::new(CheckedTerm::Int { value: 53 })
                    })
                }),
            }),
            arg: Box::new(CheckedTerm::Int { value: 42 }),
        },
        // lam y : Int. 53
        &CheckedTerm::Lam {
            param_name: "y".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Int { value: 53 })
        }
    ))
}

#[test]
fn test_subst() {
    assert_eq!(
        // lam y : Int. x (context = [x : Int = 42])
        CheckedTerm::Lam {
            param_name: "y".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Var {
                name: "x".to_string(),
                index: 1
            })
        }
        .subst(&CheckedTerm::Int { value: 42 }),
        // lam y : Int. 42
        CheckedTerm::Lam {
            param_name: "y".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Int { value: 42 })
        }
    );
    assert_eq!(
        // lam z : Int. x (context = [x : Int, y : Int = 42])
        CheckedTerm::Lam {
            param_name: "z".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Var {
                name: "x".to_string(),
                index: 2
            })
        }
        .subst(&CheckedTerm::Int { value: 42 }),
        // lam z : Int. x (context = [x : Int])
        CheckedTerm::Lam {
            param_name: "z".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body: Box::new(CheckedTerm::Var {
                name: "x".to_string(),
                index: 1
            })
        }
    );
    assert_eq!(
        // pi x : T. T (context = [T : Sort 1 = Int])
        CheckedTerm::Pi {
            param_name: "x".to_string(),
            param_type: Box::new(CheckedTerm::Var {
                name: "T".to_string(),
                index: 0
            }),
            body_type: Box::new(CheckedTerm::Var {
                name: "T".to_string(),
                index: 1
            })
        }
        .subst(&CheckedTerm::IntType),
        CheckedTerm::Pi {
            param_name: "x".to_string(),
            param_type: Box::new(CheckedTerm::IntType),
            body_type: Box::new(CheckedTerm::IntType)
        }
    );
}

pub type Context = IndexMap<String, CheckedTerm>;

fn check_sort<'src>(term: &CheckedTerm, span: &SimpleSpan) -> Result<'src, usize> {
    let CheckedTerm::Sort { level } = term else {
        return Err(vec![Error::custom(*span, "not a sort")]);
    };
    Ok(*level)
}

fn check_type_match<'src>(
    type1: &CheckedTerm,
    type2: &CheckedTerm,
    span: &SimpleSpan,
) -> Result<'src, ()> {
    if !check_beta_eq(type1, type2) {
        return Err(vec![Error::custom(
            *span,
            format!("type mismatch: {type1} and {type2}"),
        )]);
    }
    Ok(())
}

pub fn check_term<'src>(
    term: &Term,
    context: &Context,
) -> Result<'src, (CheckedTerm, CheckedTerm)> {
    match term {
        /*
         *
         * ─────────────────────────
         * Γ ⊢ Sort n : Sort (n + 1)
         */
        Term::Sort { level } => Ok((
            CheckedTerm::Sort { level: *level },
            CheckedTerm::Sort { level: level + 1 },
        )),

        /*
         *
         * ─────────────
         * Γ ⊢ () : Unit
         */
        Term::Unit => Ok((CheckedTerm::Unit, CheckedTerm::UnitType)),

        /*
         *
         * ─────────────────
         * Γ ⊢ Unit : Sort 1
         */
        Term::UnitType => Ok((CheckedTerm::UnitType, CheckedTerm::Sort { level: 1 })),

        /*
         *
         * ───────────────────
         * Γ ⊢ <integer> : Int
         */
        Term::Int { value } => Ok((CheckedTerm::Int { value: *value }, CheckedTerm::IntType)),

        /*
         *
         * ────────────────
         * Γ ⊢ Int : Sort 1
         */
        Term::IntType => Ok((CheckedTerm::IntType, CheckedTerm::Sort { level: 1 })),

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
            let (checked_param_type, param_type_type) = check_term(param_type, context)?;
            check_sort(&param_type_type, &param_type.span)?;
            let mut new_context = context.clone();
            new_context.insert(param_name.to_string(), checked_param_type.clone());
            let (checked_body, body_type) = check_term(body, &new_context)?;
            let checked_term = CheckedTerm::Lam {
                param_name: param_name.to_string(),
                param_type: Box::new(checked_param_type.clone()),
                body: Box::new(checked_body),
            };
            let ty = CheckedTerm::Pi {
                param_name: param_name.to_string(),
                param_type: Box::new(checked_param_type),
                body_type: Box::new(body_type),
            };
            Ok((checked_term, ty))
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
            let (checked_param_type, param_type_type) = check_term(param_type, context)?;
            let param_sort_level = check_sort(&param_type_type, &param_type.span)?;
            let mut new_context = context.clone();
            new_context.insert(param_name.to_string(), checked_param_type.clone());
            let (checked_body_type, body_type_type) = check_term(body_type, &new_context)?;
            let body_sort_level = check_sort(&body_type_type, &body_type.span)?;
            let checked_term = CheckedTerm::Pi {
                param_name: param_name.to_string(),
                param_type: Box::new(checked_param_type),
                body_type: Box::new(checked_body_type),
            };
            let ty = CheckedTerm::Sort {
                level: max(param_sort_level, body_sort_level),
            };
            Ok((checked_term, ty))
        }

        /*
         * x : α ∈ Γ
         * ─────────
         * Γ ⊢ x : α
         */
        Term::Var { name } => {
            let Some((index, _, ty)) = context.get_full(name.inner) else {
                return Err(vec![Error::custom(name.span, "unbound variable")]);
            };
            let checked_term = CheckedTerm::Var {
                name: name.to_string(),
                index: context.len() - 1 - index,
            };
            Ok((checked_term, ty.shift((context.len() - index) as isize, 0)))
        }

        /*
         * Γ, e₁ : α → β  Γ ⊢ e₂ : α
         * ─────────────────────────
         *   Γ ⊢ e₁ e₂ : β
         */
        Term::App { callee, arg } => {
            let (checked_callee, callee_type) = check_term(callee, context)?;
            let CheckedTerm::Pi {
                param_type,
                body_type,
                ..
            } = callee_type.whnf()
            else {
                return Err(vec![Error::custom(callee.span, "not a function")]);
            };
            let (checked_arg, arg_type) = check_term(arg, context)?;
            check_type_match(&param_type, &arg_type, &arg.span)?;
            let checked_term = CheckedTerm::App {
                callee: Box::new(checked_callee),
                arg: Box::new(checked_arg.clone()),
            };
            Ok((checked_term, body_type.subst(&checked_arg)))
        }
    }
}

pub fn check_def<'src>(
    name: &'src str,
    term: &Term<'src>,
    context: &mut Context,
) -> Result<'src, ()> {
    let (_, ty) = check_term(term, context)?;
    context.insert(name.to_string(), ty);
    Ok(())
}

pub fn check_axiom<'src>(
    name: &'src str,
    term: &Spanned<Term<'src>>,
    context: &mut Context,
) -> Result<'src, ()> {
    let (checked_term, _) = check_term(term, context)?;
    context.insert(name.to_string(), checked_term);
    Ok(())
}
