use crate::eval::eval;
use crate::expr::Expr;
use std::collections::HashMap;

pub type Env<'a> = HashMap<&'a str, Expr>;

fn collect_numbers<'a>(exprs: &'a [Expr]) -> Result<Vec<f64>, String> {
    let empty_vec: Vec<f64> = Vec::new();
    let starting: Result<Vec<f64>, String> = Ok(empty_vec);
    exprs
        .into_iter()
        .fold(starting, |acc, unevaled_expr| match acc {
            Ok(mut results_vec) => match eval(unevaled_expr) {
                Ok(Expr::Num(n)) => {
                    results_vec.push(n);
                    Ok(results_vec)
                }
                thing => Err(format!("Expected number, but found {thing:?}")),
            },
            e => e,
        })
}

pub fn get_std_lib<'a>() -> Env<'a> {
    HashMap::from([
        ("true", Expr::Boolean(true)),
        ("false", Expr::Boolean(false)),
        (
            "add",
            Expr::Proc(Box::new(|exprs| {
                collect_numbers(exprs)
                    .and_then(|xs| Ok(Expr::Num(xs.into_iter().fold(0., |a, b| a + b))))
            })),
        ),
    ])
}