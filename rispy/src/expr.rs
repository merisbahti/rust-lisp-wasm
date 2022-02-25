#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    List(Vec<Expr>),
    Num(f64),
    Keyword(String),
    Boolean(bool),
    Quote(Vec<Expr>),
}
