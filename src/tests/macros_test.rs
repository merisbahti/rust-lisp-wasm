#[test]
fn test_call_other_macro_from_macro() {
    use crate::expr::Expr;
    use crate::vm::jit_run;

    assert_eq!(
        jit_run(
            "
            (defmacro (three) 3)
            (defmacro (add) (three))
            (add)
            "
        ),
        Ok(Expr::num(3.0))
    );

    assert_eq!(
        jit_run(
            "
            (defmacro (three) 3)
            (defmacro (add) (cons '+ (cons (three) (cons (three) '())))) (add)
            "
        ),
        Ok(Expr::num(6.0))
    );
    assert_eq!(
        jit_run(
            "
            (defmacro (five) 5)
            (defmacro (add) (syntax-list '+  (five) (five))) 
            (add)
            "
        ),
        Ok(Expr::num(10.0))
    )
}

#[test]
fn macro_definition_doesnt_leak_out_of_scope() {
    use crate::vm::jit_run;
    assert_eq!(
        jit_run(
            "
            (lambda () (defmacro (three) 3) ())
            (defmacro (add) '(+ (three) (three)))
            (add)
            "
        ),
        Err("jit_run_vm:2:25: defmacro is not defined".to_string())
    )
}
