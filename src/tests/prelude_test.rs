#[test]
fn prelude_compiles() {
    let env = crate::vm::get_prelude();
    assert_eq!(env.is_ok(), true);
    assert!(
        env.map(|x| x.env.map.keys().cloned().collect::<Vec<String>>().len())
            .unwrap()
            > 10,
    );
}

#[test]
fn call_function_defined_in_prelude() {
    use crate::expr::Expr;
    let res = crate::vm::jit_run("(fold-right (lambda (x y) (+ x y)) 0 '(1 2 3 4 5))".to_string());
    assert_eq!(res, Ok(Expr::Num(15.0)));
}

#[test]
fn call_macro_defined_in_prelude() {
    use crate::expr::Expr;
    let res = crate::vm::jit_run("(cond (false 2) (true 3))".to_string());
    assert_eq!(res, Ok(Expr::Num(3.0)));
}
