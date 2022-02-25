use crate::expr::Expr;
use crate::parse::parse;

fn eval_expression<'a>(e: Expr) -> Result<Expr, String> {
    match e {
        Expr::Keyword(kw) => kw
            .parse::<f64>()
            .map(|x| Expr::Num(x))
            .or(match kw.as_str() {
                "true" => Ok(Expr::Boolean(true)),
                "false" => Ok(Expr::Boolean(false)),
                _ => Err(format!("Undefined variable: {kw}")),
            }),
        Expr::Quote(expr) => Result::Ok(Expr::List(expr)),
        Expr::Boolean(v) => Result::Ok(Expr::Boolean(v)),
        Expr::Num(n) => Result::Ok(Expr::Num(n)),
        Expr::List(_) => Result::Err("NYI".to_string()),
    }
}

#[test]
fn test_eval_primitives() {
    assert_eq!(
        eval_expression(Expr::Boolean(false)),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        eval_expression(Expr::Boolean(true)),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("false".to_string())),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("true".to_string())),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("15".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("0".to_string())),
        Ok(Expr::Num(0.0))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("15.0".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("00015.0".to_string())),
        Ok(Expr::Num(15.0))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("-00015.0".to_string())),
        Ok(Expr::Num(-15.0))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("-0.5".to_string())),
        Ok(Expr::Num(-0.5))
    );
    assert_eq!(
        eval_expression(Expr::Keyword("a".to_string())),
        Err("Undefined variable: a".to_string())
    );
    assert_eq!(
        eval_expression(Expr::Boolean(true)),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        eval_expression(Expr::Boolean(false)),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(eval_expression(Expr::Num(15.0)), Ok(Expr::Num(15.)));
    assert_eq!(eval_expression(Expr::Num(-0.5)), Ok(Expr::Num(-0.5)));
    assert_eq!(
        eval_expression(Expr::Quote(vec![Expr::Keyword("true".to_string())])),
        Ok(Expr::List(vec![Expr::Keyword("true".to_string())]))
    );
}

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse(src).and_then(|exp| eval_expression(exp).map_err(|e| e.to_string()))
}
