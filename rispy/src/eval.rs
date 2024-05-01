use crate::expr::Expr;
use crate::parse::parse;
use crate::std_env::get_std_lib;
use crate::std_env::Env;

pub fn eval_with_env(e: &Expr, env: &mut Env) -> Result<Expr, String> {
    match e {
        Expr::Quote(exprs) => Result::Ok(Expr::List(exprs.to_vec())),
        Expr::List(xs) => {
            let (head, args) = match xs.as_slice() {
                [head, args @ ..] => (head, args),
                _ => return Result::Ok(e.clone()),
            };

            match eval_with_env(head, env) {
                Ok(Expr::Proc(proc)) => proc(args, env),
                Ok(expr) => return Err(format!("Expected proc, but found: {expr}")),
                Err(e) => return Err(e),
            }
        }
        Expr::Keyword(kw) => match env.get(kw.as_str()) {
            Some(e) => Ok(e.clone()),
            None => Err(format!("Undefined variable: {kw}")),
        },
        e => Result::Ok(e.clone()),
    }
}

#[test]
fn test_eval_primitives() {
    pub fn eval(e: Expr) -> Result<Expr, String> {
        eval_with_env(&e, &mut get_std_lib())
    }
    assert_eq!(eval(Expr::Boolean(false)), Ok(Expr::Boolean(false)));
    assert_eq!(eval(Expr::Boolean(true)), Ok(Expr::Boolean(true)));
    assert_eq!(
        eval(Expr::Keyword("false".to_string())),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        eval(Expr::Keyword("true".to_string())),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(eval(Expr::Num(15.)), Ok(Expr::Num(15.0)));
    assert_eq!(eval(Expr::Num(0.)), Ok(Expr::Num(0.0)));
    assert_eq!(eval(Expr::Num(15.0)), Ok(Expr::Num(15.0)));
    assert_eq!(eval(Expr::Num(-15.0)), Ok(Expr::Num(-15.0)));
    assert_eq!(eval(Expr::Num(-0.5)), Ok(Expr::Num(-0.5)));
    assert_eq!(
        eval(Expr::Keyword("a".to_string())),
        Err("Undefined variable: a".to_string())
    );
    assert_eq!(eval(Expr::Boolean(true)), Ok(Expr::Boolean(true)));
    assert_eq!(eval(Expr::Boolean(false)), Ok(Expr::Boolean(false)));
    assert_eq!(eval(Expr::Num(15.0)), Ok(Expr::Num(15.)));
    assert_eq!(eval(Expr::Num(-0.5)), Ok(Expr::Num(-0.5)));
    assert_eq!(
        eval(Expr::Quote(vec![Expr::Keyword("true".to_string())])),
        Ok(Expr::List(vec![Expr::Keyword("true".to_string())]))
    );
}

pub fn eval_exprs(exprs: &[Expr], env: &mut Env) -> Result<Expr, String> {
    match exprs {
        [head, tail @ ..] => {
            let starting = eval_with_env(head, env);
            tail.iter().fold(starting, |acc, curr| match acc {
                Ok(_) => eval_with_env(curr, env),
                e => e,
            })
        }
        _ => Err("Expected more than 1 expr".to_string()),
    }
}

pub fn eval_from_str(src: &str) -> Result<Expr, String> {
    parse(src).and_then(|exps| eval_exprs(&exps, &mut get_std_lib()))
}

#[test]
fn test_scoping() {
    assert_eq!(
        eval_from_str(
            "
            (let a 10)
            (let b 20)
            (let f (fn (a) 
             (let b 500)
             (add a b)
            ))
            (f 1000)"
        ),
        Ok(Expr::Num(1500.))
    );
    assert_eq!(
        eval_from_str(
            "
            (let a 10)
            (let b 20)
            (let f (fn (a) 
             (let b 500)
             (add a b)
            ))
            (f 1000)
            a
            "
        ),
        Ok(Expr::Num(10.))
    );
    assert_eq!(
        eval_from_str(
            "
            (let a 10)
            (let b 20)
            (let f (fn (a) 
             (let b 500)
             (add a b)
            ))
            (f 1000)
            a
            b
            "
        ),
        Ok(Expr::Num(20.))
    )
}
#[test]
fn test_eval_fns() {
    assert_eq!(eval_from_str("(add 1 2)"), Ok(Expr::Num(3.)));
    assert_eq!(eval_from_str("(add 1 2 3 4)"), Ok(Expr::Num(10.)));
    assert_eq!(eval_from_str("()"), Ok(Expr::List(vec![])));
    assert_eq!(
        eval_from_str(
            "
            (let five 5)
            five
        "
        ),
        Ok(Expr::Num(5.))
    );
    assert_eq!(
        eval_from_str(
            "
            (let a 5)
            (add a a)
        "
        ),
        Ok(Expr::Num(10.))
    );
    assert_eq!(
        eval_from_str(
            "
            (sub 3 2)
        "
        ),
        Ok(Expr::Num(1.0))
    );
    assert_eq!(
        eval_from_str(
            "
            (let a 5)
            (let f (fn (x) (add x a)))
            (f a 10)
        "
        ),
        Err("parameter length 1 did not match argument length 2.".to_string())
    );
    assert_eq!(
        eval_from_str(
            "
            (let a 5)
            (let f (fn (x) (add x a)))
            (f 10)
        "
        ),
        Ok(Expr::Num(15.))
    );
    assert_eq!(
        eval_from_str(
            "
            (let a 5)
            (let f (fn (a) a))
            (f 200)
            a
        "
        ),
        Ok(Expr::Num(5.))
    );
    assert_eq!(
        eval_from_str(
            "(let f
                (fn (x)
                    (cond 
                        ((less x 100) (f (add 1 x))) 
                        (true x)
                    )
                )
            )
            (f 0)
        "
        ),
        Ok(Expr::Num(100.))
    );
    assert_eq!(
        eval_from_str(
            "
(let f
    (fn (x iters)
        (cond 
            ((less iters 0) x) 
            (true (f (add x x) (sub iters 1)))
        )
    )
)
(f 1 10)
        "
        ),
        Ok(Expr::Num(2048.))
    );
    assert_eq!(
        eval_from_str(
            "
(let fibiter
    (fn (n a b)
        (cond
        ((equal n 0) a)
        (true (fibiter (sub n 1) b (add a b))))))
(let fib (fn (n) (fibiter n 0 1)))
(fib 8)
        "
        ),
        Ok(Expr::Num(21.))
    );

    assert_eq!(
        eval_from_str(
            "
((fn (x) x) 10)
        "
        ),
        Ok(Expr::Num(10.))
    );
}
