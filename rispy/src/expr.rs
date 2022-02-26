use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Error;
use nom::lib::std::fmt::Formatter;

#[derive(Clone)]
pub enum Expr {
    List(Vec<Expr>),
    Num(f64),
    Keyword(String),
    Boolean(bool),
    Quote(Vec<Expr>),
    Proc(Box<fn(&[Expr]) -> Result<Expr, String>>),
}

impl Display for Expr {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::List(xs) => write!(formatter, "List()"),
            Expr::Num(x) => write!(formatter, "Num({x})"),
            Expr::Keyword(x) => write!(formatter, "Keyword()"),
            Expr::Boolean(x) => write!(formatter, "Boolean()"),
            Expr::Quote(xs) => write!(formatter, "Quote(...)"),
            Expr::Proc(xs) => write!(formatter, "Proc(..)"),
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Expr::List(xs) => write!(formatter, "List()"),
            Expr::Num(x) => write!(formatter, "Num({x})"),
            Expr::Keyword(x) => write!(formatter, "Keyword()"),
            Expr::Boolean(x) => write!(formatter, "Boolean()"),
            Expr::Quote(xs) => write!(formatter, "Quote(...)"),
            Expr::Proc(xs) => write!(formatter, "Proc(..)"),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, rhs: &Expr) -> bool {
        match (self, rhs) {
            (Expr::List(xs), Expr::List(ys)) => xs == ys,
            (Expr::Num(l), Expr::Num(r)) if l == r => true,
            (Expr::Keyword(l), Expr::Keyword(r)) if l == r => true,
            (Expr::Boolean(l), Expr::Boolean(r)) if l == r => true,
            _ => false,
        }
    }
}
