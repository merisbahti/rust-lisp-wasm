#[test]
fn test_print() {
    use crate::vm::jit_run_vm;

    fn run_and_get_log(input: &str) -> Vec<String> {
        jit_run_vm(input.to_string()).map(|x| x.log).unwrap()
    }

    assert_eq!(run_and_get_log("(display 1)"), vec!["1".to_string()]);
    assert_eq!(
        run_and_get_log("(display '(1 2))"),
        vec!["(1 2)".to_string()]
    );
    assert_eq!(
        run_and_get_log("(display (cons 1 2))"),
        vec!["(1 . 2)".to_string()]
    );
    assert_eq!(
        run_and_get_log("(display (cons 1 (cons 2 '())))"),
        vec!["(1 2)".to_string()]
    );
    assert_eq!(
        run_and_get_log("(display '(true false 1 10.5))"),
        vec!["(true false 1 10.5)".to_string()]
    );
    assert_eq!(
        run_and_get_log(
            "
            (progn 
              (display 1)
              (display 2)
              (display 3))"
        ),
        vec!["1", "2", "3"]
    );
    assert_eq!(
        run_and_get_log(r#"(print "hello" " " "there")"#),
        vec!["hello there"]
    );

    assert_eq!(
        run_and_get_log(r#"(assert (+ 1 1) 1)"#),
        vec!["assertion failed"]
    );
}
