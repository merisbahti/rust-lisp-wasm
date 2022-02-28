use crate::eval::eval_exprs;
use crate::eval::eval_with_env;
use crate::expr::Expr;
use std::collections::HashMap;
use std::sync::Arc;

pub type Env = HashMap<String, Expr>;

fn collect_numbers<'a>(exprs: &'a [Expr], env: &Env) -> Result<Vec<f64>, String> {
    let empty_vec: Vec<f64> = Vec::new();
    let starting: Result<Vec<f64>, String> = Ok(empty_vec);
    exprs.into_iter().fold(starting, |acc, unevaled_expr| {
        acc.and_then(
            |results_vec| match eval_with_env(unevaled_expr, &mut env.clone()) {
                Ok(Expr::Num(n)) => {
                    let mut clone = results_vec.clone();
                    clone.push(n);
                    Ok(clone)
                }
                Ok(thing) => Err(format!("Expected number, but found {thing:?}")),
                Err(e) => Err(e),
            },
        )
    })
}

fn collect_keywords(exprs: &[Expr]) -> Result<Vec<String>, String> {
    exprs.into_iter().fold(Ok(vec![]), |acc, maybe_kw| {
        acc.and_then(|mut results_vec| match maybe_kw {
            Expr::Keyword(kw) => {
                results_vec.push(kw.to_string());
                Ok(results_vec)
            }
            thing => Err(format!("Expected number, but found {thing:?}")),
        })
    })
}

pub fn get_std_lib() -> Env {
    HashMap::from([
        ("true".to_string(), Expr::Boolean(true)),
        ("false".to_string(), Expr::Boolean(false)),
        (
            "less".to_string(),
            Expr::Proc(Arc::new(|exprs, env| {
                collect_numbers(exprs, env).and_then(|numbers| match numbers.as_slice() {
                    [lhs, rhs] => Ok(Expr::Boolean(lhs < rhs)),
                    _ => Err("Only 2 arguments to less please".to_string()),
                })
            })),
        ),
        (
            "equal".to_string(),
            Expr::Proc(Arc::new(|exprs, env| {
                collect_numbers(exprs, env).and_then(|numbers| match numbers.as_slice() {
                    [lhs, rhs] => Ok(Expr::Boolean(lhs == rhs)),
                    _ => Err("Only 2 arguments to equal please".to_string()),
                })
            })),
        ),
        (
            "cond".to_string(),
            Expr::Proc(Arc::new(|cases, env| {
                let cond_pairs: Result<Vec<(Expr, Expr)>, String> =
                    cases.into_iter().fold(Ok(vec![]), |acc, curr| {
                        acc.and_then(|v| match curr {
                            Expr::List(l) => match l.as_slice() {
                                [head, butt] => {
                                    let mut new_acc = v.clone();
                                    new_acc.push((head.clone(), butt.clone()));
                                    Ok(new_acc)
                                }
                                _ => Err(format!("Cond expects a list of pairs ... got: {curr:?}")),
                            },
                            _ => Err(format!("Cond expects a list of pairs ... got: {curr:?}")),
                        })
                    });
                let consequent: Result<Expr, String> = cond_pairs.and_then(|res| {
                    res.into_iter()
                        .fold(None, |acc, (cond, cons)| {
                            acc.or_else(|| match eval_with_env(&cond, &mut env.clone()) {
                                Ok(Expr::Boolean(true)) => Some(
                                    eval_with_env(&cons, &mut env.clone())
                                        .map(|evaled_cons| evaled_cons),
                                ),
                                Ok(Expr::Boolean(false)) => None,
                                Err(e) => Some(Err(e)),
                                _ => Some(Err(format!(
                                    "Expected cond to revaluate to boolean, but found: {cond:?}"
                                ))),
                            })
                        })
                        .unwrap_or(Err("Nothing evaluated to true in cond".to_string()))
                });

                consequent
            })),
        ),
        (
            "add".to_string(),
            Expr::Proc(Arc::new(|exprs, env| {
                collect_numbers(exprs, env)
                    .map(|xs| Expr::Num(xs.into_iter().fold(0., |a, b| a + b)))
            })),
        ),
        (
            "sub".to_string(),
            Expr::Proc(Arc::new(|exprs, env| {
                collect_numbers(exprs, env).and_then(|xs| match xs.as_slice() {
                    [left, right] => Ok(Expr::Num(left - right)),
                    _ => Err("Sorry sub take only 2 arguments".to_string()),
                })
            })),
        ),
        (
            "let".to_string(),
            Expr::Proc(Arc::new(|exprs, env| match exprs {
                [Expr::Keyword(name), value] => eval_with_env(value, env).map(|evaled_value| {
                    env.insert(name.to_string(), evaled_value.clone());
                    evaled_value
                }),
                _ => Err("Let takes 2 arguments, a keyword and an expression".to_string()),
            })),
        ),
        (
            "fn".to_string(),
            Expr::Proc(Arc::new(|definition, _| match definition {
                [Expr::List(unparsed_parameters), proc_definition @ ..] => {
                    collect_keywords(unparsed_parameters).map(|parsed_parameters| {
                        define_proc(parsed_parameters, proc_definition.into())
                    })
                }
                _ => Err("fn takes 2 arguments, a list of arguments and a procedure".to_string()),
            })),
        ),
    ])
}

fn define_proc(parameters: Vec<String>, proc_definition: Vec<Expr>) -> Expr {
    Expr::Proc(Arc::new(move |pre_arguments, env| {
        let parameter_length = parameters.len();
        let arguments_length = pre_arguments.len();
        let mut local_env = env.clone();

        if parameter_length != arguments_length {
            return Err(format!("parameter length {parameter_length} did not match argument length {arguments_length}."));
        }

        let start: Result<Vec<Expr>, String> = Ok(vec![]);
        let res_evaled_arguments = pre_arguments.into_iter().fold(start, |acc, curr| {
            acc.and_then(|args_so_far| {
                eval_with_env(curr, &mut local_env.clone()).and_then(|evaled_arg| {
                    let mut new_vec = args_so_far.clone();
                    new_vec.push(evaled_arg);
                    Ok(new_vec)
                })
            })
        });

        res_evaled_arguments.and_then(|evaled_argumens| {
            for (k, v) in parameters.iter().zip(evaled_argumens) {
                local_env.insert(k.to_string(), v);
            }

            eval_exprs(&proc_definition, &mut local_env)
        })
    }))
}
