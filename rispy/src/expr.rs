use crate::vm::Chunk;
use crate::vm::VMInstruction;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Error;
use nom::lib::std::fmt::Formatter;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize)]
pub enum Expr {
    Pair(Box<Expr>, Box<Expr>),
    Num(f64),
    Keyword(String),
    Boolean(bool),
    Quote(Box<Expr>),
    Lambda(Chunk, Vec<String>),
    Nil,
    BuiltIn(Vec<VMInstruction>),
}

impl Display for Expr {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::Nil => write!(formatter, "'()"),
            Expr::Pair(x, y) => write!(formatter, "Pair({x:?}, {y:?})"),
            Expr::Num(x) => write!(formatter, "Num({x:?})"),
            Expr::Keyword(x) => write!(formatter, "Keyword({x:?})"),
            Expr::Boolean(x) => write!(formatter, "Boolean({x:?})"),
            Expr::Quote(xs) => write!(formatter, "Quote({xs:?})"),
            Expr::Lambda(xs, vars) => write!(formatter, "Lambda({xs:?}, {vars:?})"),
            Expr::BuiltIn(_) => write!(formatter, "BuiltIn(...)"),
        }
    }
}

#[test]
fn test_display() {
    let bool_expr = Expr::Boolean(true);
    assert_eq!(format!("{bool_expr}"), "Boolean(true)");

    let bool_expr_2 = Expr::Boolean(false);
    assert_eq!(format!("{bool_expr_2}"), "Boolean(false)");

    let empty_list = crate::parse::make_pair_from_vec(vec![]);
    assert_eq!(format!("{empty_list}"), "'()");

    let list_with_values = crate::parse::make_pair_from_vec(vec![
        Expr::Boolean(false),
        Expr::Num(5.0),
        crate::parse::make_pair_from_vec(vec![Expr::Keyword("hello".to_string())]),
    ]);
    assert_eq!(
        format!("{list_with_values}"),
        "Pair(Boolean(false), Pair(Num(5.0), Pair(Pair(Keyword(\"hello\"), '()), '())))"
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
            (Expr::Pair(ax, ay), Expr::Pair(bx, by)) => ax == bx && ay == by,
            (Expr::Num(l), Expr::Num(r)) if l == r => true,
            (Expr::Keyword(l), Expr::Keyword(r)) if l == r => true,
            (Expr::Boolean(l), Expr::Boolean(r)) if l == r => true,
            (Expr::Nil, Expr::Nil) => true,
            (Expr::Lambda(c1, s1), Expr::Lambda(c2, s2)) => c1 == c2 && s1 == s2,
            _ => false,
        }
    }
}
