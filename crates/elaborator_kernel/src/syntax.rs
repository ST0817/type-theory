pub enum ExprSyntax {
    Unit,
    UnitType,
    Nat { value: usize },
    NatType,
    Sort { level: usize },
}
