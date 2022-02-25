use crate::expr::Expr;
use crate::parse::parse;

fn eval_expression(e: &Expr) -> Result<Expr, String> {
    match e {
        Expr::Keyword(kw) => kw
            .parse::<u8>()
            .map(|x| Expr::Num(x as f64))
            .or(kw.parse::<f64>().map(|x| Expr::Num(x)))
            .or(match kw.as_str() {
                "true" => Ok(Expr::Boolean(true)),
                "false" => Ok(Expr::Boolean(false)),
                _ => Err(format!("Undefined variable: {kw}")),
            }),
        Expr::Quote(exprs) => Result::Ok(Expr::List(exprs.to_vec())),
        Expr::Boolean(v) => Result::Ok(Expr::Boolean(*v)),
        Expr::Num(n) => Result::Ok(Expr::Num(*n)),
        Expr::List(xs) => match xs.as_slice() {
            [Expr::Keyword(proc_name), args @ ..] => apply_proc(proc_name, args),
            _ => Result::Err("Cannot evaluate empty list".to_string()),
        },
    }
}

fn collect_numbers<'a>(exprs: &'a [Expr]) -> Result<Vec<f64>, String> {
    let empty_vec: Vec<f64> = Vec::new();
    let starting: Result<Vec<f64>, String> = Ok(empty_vec);
    exprs
        .into_iter()
        .fold(starting, |acc, unevaled_expr| match acc {
            Ok(mut results_vec) => match eval_expression(unevaled_expr) {
                Ok(Expr::Num(n)) => {
                    results_vec.push(n);
                    Ok(results_vec)
                }
                thing => Err(format!("Expected number, but found {thing:?}")),
            },
            e => e,
        })
}

fn apply_proc<'a>(name: &str, args: &[Expr]) -> Result<Expr, String> {
    match name {
        "add" => collect_numbers(args)
            .and_then(|xs| Ok(Expr::Num(xs.into_iter().fold(0., |a, b| a + b)))),
        _ => Err("haha".to_string()),
    }
}

#[test]
fn test_eval_primitives() {
    assert_eq!(
        eval_expression(&Expr::Boolean(false)),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        eval_expression(&Expr::Boolean(true)),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("false".to_string())),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("true".to_string())),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("15".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("0".to_string())),
        Ok(Expr::Num(0.0))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("15.0".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("00015.0".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("-00015.0".to_string())),
        Ok(Expr::Num(-15.0))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("-0.5".to_string())),
        Ok(Expr::Num(-0.5))
    );
    assert_eq!(
        eval_expression(&Expr::Keyword("a".to_string())),
        Err("Undefined variable: a".to_string())
    );
    assert_eq!(
        eval_expression(&Expr::Boolean(true)),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        eval_expression(&Expr::Boolean(false)),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(eval_expression(&Expr::Num(15.0)), Ok(Expr::Num(15.)));
    assert_eq!(eval_expression(&Expr::Num(-0.5)), Ok(Expr::Num(-0.5)));
    assert_eq!(
        eval_expression(&Expr::Quote(vec![Expr::Keyword("true".to_string())])),
        Ok(Expr::List(vec![Expr::Keyword("true".to_string())]))
    );
}

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse(src).and_then(|exp| eval_expression(&exp).map_err(|e| e.to_string()))
}

#[test]
fn test_eval_fns() {
    assert_eq!(eval_from_str("(add 1 2)"), Ok(Expr::Num(3.)));
    assert_eq!(eval_from_str("(add 1 2 3 4)"), Ok(Expr::Num(10.)));
    assert_eq!(
        eval_from_str("()"),
        Err("Cannot evaluate empty list".to_string())
    );
}
