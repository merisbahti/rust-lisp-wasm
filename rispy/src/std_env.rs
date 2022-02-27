use crate::eval::eval_with_env;
use crate::expr::Expr;
use std::collections::HashMap;

pub type Env = HashMap<String, Expr>;

fn collect_numbers<'a>(exprs: &'a [Expr], env: Env) -> Result<Vec<f64>, String> {
    let empty_vec: Vec<f64> = Vec::new();
    let starting: Result<Vec<f64>, String> = Ok(empty_vec);
    exprs
        .into_iter()
        .fold(starting, |acc, unevaled_expr| match acc {
            Ok(mut results_vec) => match eval_with_env(unevaled_expr, env.clone()) {
                Ok((Expr::Num(n), _)) => {
                    results_vec.push(n);
                    Ok(results_vec)
                }
                thing => Err(format!("Expected number, but found {thing:?}")),
            },
            e => e,
        })
}

pub fn get_std_lib() -> Env {
    HashMap::from([
        ("true".to_string(), Expr::Boolean(true)),
        ("false".to_string(), Expr::Boolean(false)),
        (
            "add".to_string(),
            Expr::Proc(Box::new(|exprs, env| {
                collect_numbers(exprs, env.clone())
                    .and_then(|xs| Ok((Expr::Num(xs.into_iter().fold(0., |a, b| a + b)), env)))
            })),
        ),
        (
            "let".to_string(),
            Expr::Proc(Box::new(|exprs, env| {
                let mut copied_env = env.clone();
                match exprs {
                    [Expr::Keyword(name), value] => {
                        eval_with_env(value, env).map(|(evaled_value, _)| {
                            copied_env.insert(name.to_string(), evaled_value.clone());
                            (evaled_value, copied_env)
                        })
                    }
                    _ => Err("Let takes 2 arguments, a keyword and an expression".to_string()),
                }
            })),
        ),
    ])
}
