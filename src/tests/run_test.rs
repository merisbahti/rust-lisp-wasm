#[test]
fn many_tests() {
    use crate::vm::jit_run;
    assert_eq!(
        jit_run(
            "
        (define a '(1 2 3 4))
        (apply + a)            
        ",
        ),
        Ok(crate::expr::Expr::Num(10.0))
    );

    assert_eq!(
        jit_run(
            "
        (define (fn a b) (+ a b))
        (define a '(1 2))
        (apply fn a)            
        ",
        ),
        Ok(crate::expr::Expr::Num(3.0))
    );

    assert_eq!(
        jit_run(
            "
        (define someval 10)
        (define (fn a b) (+ a b someval))
        (define a '(1 2))
        (apply fn a)            
        ",
        ),
        Ok(crate::expr::Expr::Num(13.0))
    );

    assert_eq!(
        jit_run(
            "
        (define someval 10)
        (define a '(1 2))
        (apply (lambda (a b) (+ a b someval)) a)            
        ",
        ),
        Ok(crate::expr::Expr::Num(13.0))
    );

    assert_eq!(
        jit_run(
            "
        (define someval 10)
        (define a '(1 2))
        (apply (lambda (a b) (+ a b someval)) a)            
        ",
        ),
        Ok(crate::expr::Expr::Num(13.0))
    );

    assert_eq!(
        jit_run(
            "
        (apply (lambda (a b) b) '(0 15))            
        ",
        ),
        Ok(crate::expr::Expr::Num(15.0))
    );

    assert_eq!(
        jit_run(
            "
        (apply (lambda (a b) a) '(0 15))            
        ",
        ),
        Ok(crate::expr::Expr::Num(0.0))
    );
}
