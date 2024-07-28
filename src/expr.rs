use crate::vm::Chunk;
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
    String(String),
    Quote(Box<Expr>),
    Lambda(
        Chunk,
        Vec<String>,
        Option<String>, /* variadic */
        String,         /* env where it was defined*/
    ),
    LambdaDefinition(Chunk, Option<String> /* variadic? */, Vec<String>),
    Nil,
}

impl Display for Expr {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::Nil => write!(formatter, "'()"),
            Expr::Pair(x, box Expr::Nil) => write!(formatter, "({x:?})"),
            Expr::Pair(x, r @ box Expr::Pair(_, _)) => {
                let mut r_string = format!("{r}");
                // remove parens for niceness
                r_string.pop();
                r_string.remove(0);
                write!(formatter, "({x:?} {r_string})")
            }
            Expr::Pair(x, y) => write!(formatter, "({x:?} . {y:?})"),
            Expr::Num(x) => {
                let mut string_value = format!("{:#?}", x);
                if string_value.ends_with(".0") {
                    string_value.pop();
                    string_value.pop();
                    write!(formatter, "{}", string_value)
                } else {
                    write!(formatter, "{:#?}", x)
                }
            }
            Expr::Keyword(x) => write!(formatter, "{x}"),
            Expr::Boolean(x) => write!(formatter, "{x:?}"),
            Expr::Quote(xs) => write!(formatter, "'{xs:?}"),
            Expr::Lambda(xs, vars, variadic, env) => {
                write!(formatter, "Lambda({xs:?}, {vars:?}, {variadic:?}, {env:?})")
            }
            Expr::String(s) => {
                write!(formatter, "string {s:?}")
            }
            Expr::LambdaDefinition(xs, variadic, vars) => {
                write!(
                    formatter,
                    "LambdaDefinition({xs:?}, {variadic:?}, {vars:?})"
                )
            }
        }
    }
}

#[test]
fn test_display() {
    let bool_expr = Expr::Boolean(true);
    assert_eq!(format!("{bool_expr}"), "true");

    let bool_expr_2 = Expr::Boolean(false);
    assert_eq!(format!("{bool_expr_2}"), "false");

    let empty_list = crate::parse::make_pair_from_vec(vec![]);
    assert_eq!(format!("{empty_list}"), "'()");

    let list_with_values = crate::parse::make_pair_from_vec(vec![
        Expr::Boolean(false),
        Expr::Num(5.0),
        crate::parse::make_pair_from_vec(vec![Expr::Keyword("hello".to_string())]),
    ]);
    assert_eq!(format!("{list_with_values}"), "(false 5 (hello))");
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
            (Expr::String(l), Expr::String(r)) if l == r => true,
            (Expr::Keyword(l), Expr::Keyword(r)) if l == r => true,
            (Expr::Boolean(l), Expr::Boolean(r)) if l == r => true,
            (Expr::Nil, Expr::Nil) => true,
            (Expr::Lambda(c1, s1, variadic1, d1), Expr::Lambda(c2, s2, variadic2, d2)) => {
                c1 == c2 && s1 == s2 && d1 == d2 && variadic1 == variadic2
            }
            (Expr::LambdaDefinition(c1, v1, s1), Expr::LambdaDefinition(c2, v2, s2)) => {
                c1 == c2 && s1 == s2 && v1 == v2
            }
            _ => false,
        }
    }
}
