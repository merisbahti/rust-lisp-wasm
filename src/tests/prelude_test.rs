#[test]
fn prelude_compiles() {
    let env = crate::vm::get_prelude();
    assert!(env.is_ok());
    assert!(
        env.map(|x| x.env.map.keys().cloned().collect::<Vec<String>>().len())
            .unwrap()
            > 10,
    );
}

#[test]
fn call_function_defined_in_prelude() {
    use crate::expr::Expr;
    let res = crate::vm::jit_run("(fold-right (lambda (x y) (+ x y)) 0 '(1 2 3 4 5))");
    assert_eq!(res, Ok(Expr::num(15.0)));
}

#[test]
fn call_macro_defined_in_prelude() {
    use crate::expr::Expr;
    let res = crate::vm::jit_run("(cond (false 2) (true 3))");
    assert_eq!(res, Ok(Expr::num(3.0)));
}

#[test]
fn assert_test() {
    let res = crate::vm::jit_run_vm(
        "
        (assert (+ 1 1) 2)
        (assert (+ 1 1) 3)
        ",
    )
    .unwrap()
    .log;
    assert_eq!(
        res,
        vec!["assertion failed, found: 2 but expected: 3. (+ 1 1) != 3".to_string()]
    );
}
