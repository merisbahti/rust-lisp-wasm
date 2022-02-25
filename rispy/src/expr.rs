#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    List(Vec<Expr>),
    Num(i32),
    Keyword(String),
    Boolean(bool),
    Quote(Vec<Expr>),
}
