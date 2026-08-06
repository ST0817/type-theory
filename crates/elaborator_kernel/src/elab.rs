use crate::{expr::Expr, syntax::ExprSyntax};

pub fn elab_expr_syntax(syntax: &ExprSyntax) -> Expr {
    match syntax {
        ExprSyntax::Unit => Expr::Unit,
        ExprSyntax::UnitType => Expr::UnitType,
        ExprSyntax::Nat { value } => Expr::Nat { value: *value },
        ExprSyntax::NatType => Expr::NatType,
        ExprSyntax::Sort { level } => Expr::Sort { level: *level },
    }
}
