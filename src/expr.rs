use crate::parse::SrcLoc;
use crate::vm::Chunk;
use core::fmt::Debug;
use core::fmt::Display;

#[derive(Clone, Debug)]
pub struct Num {
    pub value: f64,
    pub srcloc: Option<SrcLoc>,
}

#[derive(Clone, Debug)]
pub struct Bool {
    pub value: bool,
    pub srcloc: Option<SrcLoc>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Pair(Box<Expr>, Box<Expr>, Option<SrcLoc>),
    Num(Num),
    Keyword(String, Option<SrcLoc>),
    Boolean(Bool),
    String(String, Option<SrcLoc>),
    Quote(Box<Expr>, Option<SrcLoc>),
    Lambda(
        Chunk,
        Vec<String>,
        Option<String>, /* variadic */
        String,         /* env where it was defined*/
    ),
    Nil,
}

impl Expr {
    pub fn num(value: f64) -> Self {
        Self::Num(Num {
            value,
            srcloc: None,
        })
    }
    pub fn bool(value: bool) -> Self {
        Self::Boolean(Bool {
            value,
            srcloc: None,
        })
    }
}

impl Display for Expr {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Expr::Nil => write!(formatter, "'()"),
            Expr::Pair(x, box Expr::Nil, ..) => write!(formatter, "({x})"),
            Expr::Pair(x, r @ box Expr::Pair(..), ..) => {
                let mut r_string = format!("{r}");
                // remove parens for niceness
                r_string.pop();
                r_string.remove(0);
                write!(formatter, "({x} {r_string})")
            }
            Expr::Pair(x, y, ..) => write!(formatter, "({x} . {y})"),
            Expr::Num(Num { value: x, .. }) => {
                let mut string_value = format!("{}", x);
                if string_value.ends_with(".0") {
                    string_value.pop();
                    string_value.pop();
                    write!(formatter, "{}", string_value)
                } else {
                    write!(formatter, "{}", x)
                }
            }
            Expr::Keyword(x, ..) => write!(formatter, "{x}"),
            Expr::Boolean(Bool { value: x, .. }) => write!(formatter, "{x}"),
            Expr::Quote(xs, _) => write!(formatter, "'{xs:?}"),
            Expr::Lambda(..) => {
                write!(formatter, "Lambda(...)")
            }
            Expr::String(s, _) => {
                write!(formatter, "{s}")
            }
        }
    }
}

#[test]
fn test_display() {
    let bool_expr = Expr::bool(true);
    assert_eq!(format!("{bool_expr}"), "true");

    let bool_expr_2 = Expr::bool(false);
    assert_eq!(format!("{bool_expr_2}"), "false");

    let empty_list = crate::parse::make_pair_from_vec(vec![]);
    assert_eq!(format!("{empty_list}"), "'()");

    let list_with_values = crate::parse::make_pair_from_vec(vec![
        Expr::bool(false),
        Expr::num(5.0),
        crate::parse::make_pair_from_vec(vec![Expr::Keyword("hello".to_string(), None)]),
    ]);
    assert_eq!(format!("{list_with_values}"), "(false 5 (hello))");
}

impl PartialEq for Expr {
    fn eq(&self, rhs: &Expr) -> bool {
        match (self, rhs) {
            (Expr::Pair(ax, ay, ..), Expr::Pair(bx, by, ..)) => ax == bx && ay == by,
            (Expr::Num(Num { value: l, .. }), Expr::Num(Num { value: r, .. })) if l == r => true,
            (Expr::String(l, _), Expr::String(r, _)) if l == r => true,
            (Expr::Keyword(l, ..), Expr::Keyword(r, ..)) if l == r => true,
            (Expr::Boolean(Bool { value: l, .. }), Expr::Boolean(Bool { value: r, .. }))
                if l == r =>
            {
                true
            }
            (Expr::Nil, Expr::Nil) => true,
            (Expr::Lambda(c1, s1, variadic1, d1), Expr::Lambda(c2, s2, variadic2, d2)) => {
                c1 == c2 && s1 == s2 && d1 == d2 && variadic1 == variadic2
            }
            _ => false,
        }
    }
}
