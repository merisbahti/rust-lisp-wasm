#[cfg(test)]
use std::assert_matches::assert_matches;

#[cfg(test)]
use crate::comp_err;
#[cfg(test)]
use crate::compile::compile_internal;
#[cfg(test)]
use crate::compile::find_closed_vars_in_fn;
#[cfg(test)]
use crate::compile::get_all_defines;
#[cfg(test)]
use crate::compile::{compile_many_exprs, CompileError};
#[cfg(test)]
use crate::parse::SrcLoc;
#[cfg(test)]
use crate::{
    expr::Expr,
    vm::{Chunk, VMInstruction},
};
#[test]
fn test_simple_add_compilation() {
    let mut initial_chunk = Chunk { code: vec![] };

    match compile_internal(
        &crate::parse::make_pair_from_vec(vec![
            Expr::Keyword("+".to_string(), None),
            Expr::num(1.0),
            Expr::num(2.0),
        ]),
        &mut initial_chunk,
        &mut vec![],
    ) {
        Ok(_) => {}
        Err(e) => panic!("Error {:?}", e),
    };
    assert_eq!(
        initial_chunk,
        Chunk {
            code: vec![
                VMInstruction::Lookup("+".to_string()),
                VMInstruction::Constant(Expr::num(1.0)),
                VMInstruction::Constant(Expr::num(2.0)),
                VMInstruction::Call(2),
            ],
        }
    )
}

#[test]
fn losta_compile() {
    fn parse_and_compile(input: &str) -> Vec<VMInstruction> {
        let expr = crate::parse::parse(&crate::parse::ParseInput {
            source: input,
            file_name: Some("parse_and_compile"),
        })
        .unwrap()
        .first()
        .unwrap()
        .clone();
        let mut chunk = Chunk { code: vec![] };
        match compile_internal(
            &expr,
            &mut chunk,
            &mut vec!["get".to_string(), "add".to_string()],
        ) {
            Ok(..) => chunk.code,
            Err(e) => panic!("Error when compiling {:?}: {:?}", input, e),
        }
    }

    assert_eq!(
        parse_and_compile("(+ 1 2)"),
        vec![
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::Constant(Expr::num(2.0)),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(
        crate::vm::prepare_vm(
            &crate::parse::ParseInput {
                source: "(+ 1 2 3)",
                file_name: Some("parse_and_compile"),
            },
            None
        )
        .map(|x| x
            .0
            .callframes
            .first()
            .map(|x| x.chunk.code.clone())
            .unwrap()),
        Ok(vec![
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::Constant(Expr::num(2.0)),
            VMInstruction::Constant(Expr::num(3.0)),
            VMInstruction::Call(3),
            VMInstruction::Return,
        ])
    );

    assert_eq!(
        parse_and_compile("((get add) 1 2 3)"),
        vec![
            VMInstruction::Lookup("get".to_string()),
            VMInstruction::Lookup("add".to_string()),
            VMInstruction::Call(1),
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::Constant(Expr::num(2.0)),
            VMInstruction::Constant(Expr::num(3.0)),
            VMInstruction::Call(3),
        ]
    );

    assert_eq!(
        parse_and_compile("((get add) (+ 1 2) 3)"),
        vec![
            VMInstruction::Lookup("get".to_string()),
            VMInstruction::Lookup("add".to_string()),
            VMInstruction::Call(1),
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::Constant(Expr::num(2.0)),
            VMInstruction::Call(2),
            VMInstruction::Constant(Expr::num(3.0)),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(
        parse_and_compile("()"),
        vec![VMInstruction::Constant(Expr::Nil)]
    );

    assert_eq!(
        parse_and_compile("(get)"),
        vec![
            VMInstruction::Lookup("get".to_string()),
            VMInstruction::Call(0)
        ]
    );

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        vec![
            VMInstruction::MakeLambda(
                Chunk {
                    code: vec![
                        VMInstruction::Constant(Expr::num(1.0)),
                        VMInstruction::Return
                    ],
                },
                None,
                vec![],
                vec![],
                vec![],
            ),
            VMInstruction::Call(0)
        ]
    );
    assert_eq!(
        parse_and_compile("(define a 1)"),
        vec![
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::Define("a".to_string()),
            VMInstruction::Constant(Expr::Nil),
        ]
    );
}

#[test]
fn compile_recursive() {
    fn parse_and_compile(input: &str) -> Result<(), crate::compile::CompileError> {
        let expr = crate::parse::parse(&crate::parse::ParseInput {
            source: input,
            file_name: Some("lambda_compile_test"),
        })
        .map_err(|err| CompileError {
            srcloc: None,
            message: err,
        })?;
        let mut chunk = Chunk { code: vec![] };
        compile_many_exprs(expr, &mut chunk, &mut vec![])
    }

    assert_matches!(
        parse_and_compile(
            "
        (define (f x) (f x))"
        ),
        Ok(..)
    );
    assert_matches!(
        parse_and_compile(
            "
        (define (f x) (f))"
        ),
        Ok(..)
    );
    assert_matches!(
        parse_and_compile(
            "
        (define (list . xs) (xs))"
        ),
        Ok(..)
    );
    assert_matches!(
        parse_and_compile(
            "
        (define (list . xs) (xz))"
        ),
        Err(CompileError {
            message,
            ..
        }) if message == *"xz is not defined"
    );
    assert_matches!(
        parse_and_compile(
            "
(define (fold-right op initial sequence)
    (if

      (nil? sequence)
      initial
      (op
        (car sequence)
        (fold-right op initial (cdr sequence)))))
"
        ),
        Ok(..)
    );
}

#[test]
fn lambda_compile_test() {
    fn parse_and_compile(input: &str) -> Chunk {
        let expr = crate::parse::parse(&crate::parse::ParseInput {
            source: input,
            file_name: Some("lambda_compile_test"),
        })
        .unwrap()
        .first()
        .unwrap()
        .clone();
        let mut chunk = Chunk { code: vec![] };
        match compile_internal(&expr, &mut chunk, &mut vec![]) {
            Ok(()) => chunk,
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    assert_eq!(
        parse_and_compile("(if 1 2 3)").code,
        vec![
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::CondJumpPop(3),
            VMInstruction::Constant(Expr::num(3.0)),
            VMInstruction::Constant(Expr::bool(true)),
            VMInstruction::CondJumpPop(1),
            VMInstruction::Constant(Expr::num(2.0)),
        ]
    );
    assert_eq!(
        parse_and_compile("(lambda () 1)"),
        Chunk {
            code: vec![VMInstruction::MakeLambda(
                Chunk {
                    code: vec![
                        VMInstruction::Constant(Expr::num(1.0)),
                        VMInstruction::Return
                    ],
                },
                None,
                vec![],
                vec![],
                vec![],
            )],
        }
    );

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        Chunk {
            code: vec![
                VMInstruction::MakeLambda(
                    Chunk {
                        code: vec![
                            VMInstruction::Constant(Expr::num(1.0)),
                            VMInstruction::Return
                        ],
                    },
                    None,
                    vec![],
                    vec![],
                    vec![],
                ),
                VMInstruction::Call(0)
            ],
        }
    );
}

#[test]
fn local_vars_test() {
    fn parse_and_close(input: &str) -> Result<Vec<String>, CompileError> {
        let parsed = crate::parse::parse(&crate::parse::ParseInput {
            source: input,
            file_name: Some("lambda_compile_test"),
        })
        .map_err(|err| CompileError {
            srcloc: None,
            message: err,
        })?;

        Ok(get_all_defines(&parsed))
    }

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(define c 12)
(define (f q z) 12)
(define b 30)
(define q)
(define (g) 12)
",
        ),
        Ok(vec!["a", "c", "f", "b", "q", "g"]
            .into_iter()
            .map(|x| x.to_string())
            .collect())
    );

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(lambda (x) ; lambda closing over a
  (define m 1)
  (define (g y) ; lambda closing over a, b, x
    (+ a b x y)))
(define b 30)
",
        ),
        Ok(vec!["a", "b"].into_iter().map(|x| x.to_string()).collect())
    );
}
#[test]
fn close_variables_test() {
    fn parse_and_close(input: &str) -> Result<Vec<String>, CompileError> {
        let parsed = crate::parse::parse(&crate::parse::ParseInput {
            source: input,
            file_name: Some("close_test"),
        })
        .map_err(|err| CompileError {
            srcloc: None,
            message: err,
        })?;

        let parent_variables = get_all_defines(&parsed);
        match parsed.last() {
            Some(Expr::Pair(
                box Expr::Keyword(lambda_kw, ..),
                box Expr::Pair(kw_pairs, box lambda_body, ..),
                ..,
            )) if *lambda_kw == "lambda".to_string() => {
                find_closed_vars_in_fn(&parent_variables, kw_pairs, lambda_body)
            }
            Some(Expr::Pair(
                box Expr::Keyword(define_kw, ..),
                box Expr::Pair(
                    box Expr::Pair(box Expr::Keyword(_lambda_name, ..), kw_pairs, ..),
                    box lambda_body,
                    ..,
                ),
                ..,
            )) if *define_kw == "define".to_string() => {
                find_closed_vars_in_fn(&parent_variables, kw_pairs, lambda_body)
            }
            Some(last) => {
                comp_err!(
                    last,
                    "expected lambda as last expr in test, but found this."
                )
            }
            None => comp_err!(&Expr::Nil, "no expr compiled from: {input}"),
        }
    }

    assert_eq!(
        parse_and_close(
            "
(lambda (x) ())
",
        ),
        Ok(vec![])
    );

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(lambda (x) a)
",
        ),
        Ok(vec!["a".to_string()])
    );
    assert_eq!(
        parse_and_close(
            "
(define a 10)
(define (f x) a)
",
        ),
        Ok(vec!["a".to_string()])
    );

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(lambda (x) (a x))
",
        ),
        Ok(vec!["a".to_string()])
    );

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(lambda (a) (+ a 1))
",
        ),
        Ok(vec![])
    );
    assert_eq!(
        parse_and_close(
            "
(define a 10)
(define (f x) (a x))
",
        ),
        Ok(vec!["a".to_string()])
    );

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(lambda (x) ; lambda closing over a
  (define b 1)
  (define (g z) ; lambda def
      z)  
  (lambda (x y) ; lambda closing over a, b, x
    (+ a b x (g y))))
",
        ),
        Ok(vec!["a".to_string()])
    );

    assert_eq!(
        parse_and_close(
            "
(define a 10)
(define (f x) ; lambda closing over a
  (define b 1)
  (define (g y) ; lambda closing over a, b, x
    (+ a b x y not_defined)))
",
        ),
        Err(CompileError {
            srcloc: Some(SrcLoc {
                line: 6,
                column: 16,
                file_name: Some("close_test".to_string())
            }),
            message: "not_defined is not defined".to_string()
        })
    );
}
