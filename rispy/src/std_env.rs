use crate::expr::Expr;
use std::collections::HashMap;

pub type Env<'a> = HashMap<&'a str, Expr>;

pub fn get_std_lib<'a>() -> Env<'a> {
    HashMap::from([
        ("true", Expr::Boolean(true)),
        ("false", Expr::Boolean(false)),
    ])
}
