use crate::expr::Expr;

pub fn infer_type(expr: &Expr) -> Expr {
    match expr {
        Expr::Unit => Expr::UnitType,
        Expr::UnitType => Expr::Sort { level: 1 },
        Expr::Nat { .. } => Expr::NatType,
        Expr::NatType => Expr::Sort { level: 1 },
        Expr::Sort { level } => Expr::Sort { level: level + 1 },
    }
}
