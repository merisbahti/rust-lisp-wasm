use crate::expr::Expr;
use crate::parse::parse;
use crate::std_env::get_std_lib;
use crate::std_env::Env;

pub fn eval(e: &Expr) -> Result<Expr, String> {
    eval_with_env(e, get_std_lib()).map(|(expr, _)| expr)
}

pub fn eval_with_env(e: &Expr, env: Env) -> Result<(Expr, Env), String> {
    match e {
        Expr::Keyword(kw) => kw
            .parse::<u8>()
            .map(|x| (Expr::Num(x as f64), env.clone()))
            .or(kw.parse::<f64>().map(|x| (Expr::Num(x), env.clone())))
            .or(match env.get(kw.as_str()) {
                Some(e) => Ok((e.clone(), env)),
                None => Err(format!("Undefined variable: {kw}")),
            }),
        Expr::Quote(exprs) => Result::Ok((Expr::List(exprs.to_vec()), env)),
        Expr::Boolean(v) => Result::Ok((Expr::Boolean(*v), env)),
        Expr::Num(n) => Result::Ok((Expr::Num(*n), env)),
        Expr::Proc(_) => Result::Err("Can not eval proc.".to_string()),
        Expr::List(xs) => match xs.as_slice() {
            [Expr::Keyword(proc_name), args @ ..] => {
                let gotten = env.get(proc_name.as_str());
                match gotten {
                    Some(Expr::Proc(proc)) => proc(args, env.clone()),
                    Some(expr) => Err(format!("Cannot eval {expr}")),
                    None => Err(format!("Cannot find {proc_name}")),
                }
            }
            _ => Result::Err("Cannot evaluate empty list".to_string()),
        },
    }
}

#[test]
fn test_eval_primitives() {
    assert_eq!(eval(&Expr::Boolean(false)), Ok(Expr::Boolean(false)));
    assert_eq!(eval(&Expr::Boolean(true)), Ok(Expr::Boolean(true)));
    assert_eq!(
        eval(&Expr::Keyword("false".to_string())),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        eval(&Expr::Keyword("true".to_string())),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(eval(&Expr::Keyword("15".to_string())), Ok(Expr::Num(15.0)));
    assert_eq!(eval(&Expr::Keyword("0".to_string())), Ok(Expr::Num(0.0)));
    assert_eq!(
        eval(&Expr::Keyword("15.0".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval(&Expr::Keyword("00015.0".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval(&Expr::Keyword("-00015.0".to_string())),
        Ok(Expr::Num(-15.0))
    );
    assert_eq!(
        eval(&Expr::Keyword("-0.5".to_string())),
        Ok(Expr::Num(-0.5))
    );
    assert_eq!(
        eval(&Expr::Keyword("a".to_string())),
        Err("Undefined variable: a".to_string())
    );
    assert_eq!(eval(&Expr::Boolean(true)), Ok(Expr::Boolean(true)));
    assert_eq!(eval(&Expr::Boolean(false)), Ok(Expr::Boolean(false)));
    assert_eq!(eval(&Expr::Num(15.0)), Ok(Expr::Num(15.)));
    assert_eq!(eval(&Expr::Num(-0.5)), Ok(Expr::Num(-0.5)));
    assert_eq!(
        eval(&Expr::Quote(vec![Expr::Keyword("true".to_string())])),
        Ok(Expr::List(vec![Expr::Keyword("true".to_string())]))
    );
}

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse(src).and_then(|exp| eval(&exp).map_err(|e| e.to_string()))
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
