use crate::eval::eval_exprs;
use crate::eval::eval_with_env;
use crate::expr::Expr;
use std::collections::HashMap;
use std::sync::Arc;

pub type Env = HashMap<String, Expr>;

fn collect_numbers<'a>(exprs: &'a [Expr], env: &Env) -> Result<Vec<f64>, String> {
    let empty_vec: Vec<f64> = Vec::new();
    let starting: Result<Vec<f64>, String> = Ok(empty_vec);
    exprs
        .into_iter()
        .fold(starting, |acc, unevaled_expr| match acc {
            Ok(mut results_vec) => match eval_with_env(unevaled_expr, env) {
                Ok((Expr::Num(n), _)) => {
                    results_vec.push(n);
                    Ok(results_vec)
                }
                thing => Err(format!("Expected number, but found {thing:?}")),
            },
            e => e,
        })
}

fn collect_keywords(exprs: &[Expr]) -> Result<Vec<String>, String> {
    exprs
        .into_iter()
        .fold(Ok(vec![]), |acc, maybe_kw| match acc {
            Ok(mut results_vec) => match maybe_kw {
                Expr::Keyword(kw) => {
                    results_vec.push(kw.to_string());
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
            Expr::Proc(Arc::new(|exprs, env| {
                collect_numbers(exprs, env).and_then(|xs| {
                    Ok((
                        Expr::Num(xs.into_iter().fold(0., |a, b| a + b)),
                        env.clone(),
                    ))
                })
            })),
        ),
        (
            "let".to_string(),
            Expr::Proc(Arc::new(|exprs, env| {
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
        (
            "fn".to_string(),
            Expr::Proc(Arc::new(|definition, env| match definition {
                [Expr::List(unparsed_parameters), proc_definition @ ..] => {
                    collect_keywords(unparsed_parameters).map(|parsed_parameters| {
                        (
                            define_proc(parsed_parameters, proc_definition.into()),
                            env.clone(),
                        )
                    })
                }
                _ => Err("Let takes 2 arguments, a list of arguments and a procedure".to_string()),
            })),
        ),
    ])
}

fn define_proc(parameters: Vec<String>, proc_definition: Vec<Expr>) -> Expr {
    Expr::Proc(Arc::new(move |pre_arguments, env| {
        let parameter_length = parameters.len();
        let arguments_length = pre_arguments.len();
        if parameter_length != arguments_length {
            return Err(format!("parameter length {parameter_length} did not match argument length {arguments_length}."));
        }
        let start: Result<Vec<Expr>, String> = Ok(vec![]);
        let res_evaled_arguments = pre_arguments.into_iter().fold(start, move |acc, curr| {
            acc.and_then(move |args_so_far| {
                eval_with_env(curr, env).and_then(|(evaled_arg, _)| {
                    let mut new_vec = args_so_far.clone();
                    new_vec.push(evaled_arg);
                    Ok(new_vec)
                })
            })
        });

        res_evaled_arguments.and_then(|evaled_argumens| {
            let mut local_env: Env = env.clone();
            for (k, v) in parameters.iter().zip(evaled_argumens) {
                local_env.insert(k.to_string(), v);
            }

            eval_exprs(&proc_definition, local_env).map(|(expr, _)| (expr, env.clone()))
        })
    }))
}
