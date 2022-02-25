use crate::expr::Expr;
use crate::parse::parse;

fn eval_expression<'a>(e: Expr) -> Result<Expr, &'a str> {
    match e {
        Expr::Num(_) | Expr::Quote(_) => Result::Ok(e),
        _ => Result::Err("Sorry something bad happened"),
    }
}

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse(src).and_then(|exp| eval_expression(exp).map_err(|e| e.to_string()))
}
