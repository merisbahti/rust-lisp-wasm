use crate::std_env::Env;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Error;
use nom::lib::std::fmt::Formatter;
use std::sync::Arc;

#[derive(Clone)]
pub enum Expr {
    List(Vec<Expr>),
    Num(f64),
    Keyword(String),
    Boolean(bool),
    Quote(Vec<Expr>),
    Proc(Arc<dyn Fn(&[Expr], &Env) -> Result<(Expr, Env), String>>),
}

impl Display for Expr {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::List(x) => write!(formatter, "List({x:?})"),
            Expr::Num(x) => write!(formatter, "Num({x:?})"),
            Expr::Keyword(x) => write!(formatter, "Keyword({x:?})"),
            Expr::Boolean(x) => write!(formatter, "Boolean({x:?})"),
            Expr::Quote(xs) => write!(formatter, "Quote({xs:?})"),
            Expr::Proc(_) => write!(formatter, "Proc(...)"),
        }
    }
}

#[test]
fn test_display() {
    let bool_expr = Expr::Boolean(true);
    assert_eq!(format!("{bool_expr}"), "Boolean(true)");

    let bool_expr_2 = Expr::Boolean(false);
    assert_eq!(format!("{bool_expr_2}"), "Boolean(false)");

    let empty_list = Expr::List(vec![]);
    assert_eq!(format!("{empty_list}"), "List([])");

    let list_with_values = Expr::List(vec![
        Expr::Boolean(false),
        Expr::Num(5.0),
        Expr::List(vec![Expr::Keyword("hello".to_string())]),
    ]);
    assert_eq!(
        format!("{list_with_values}"),
        "List([Boolean(false), Num(5.0), List([Keyword(\"hello\")])])"
    );
}

impl Debug for Expr {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(formatter, "{self}")
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
