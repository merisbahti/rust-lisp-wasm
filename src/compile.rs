use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use crate::{
    expr::Expr,
    parse::make_pair_from_vec,
    vm::{Chunk, VMInstruction},
};

pub enum BuiltIn {
    OneArg(fn(&Expr) -> Result<Expr, String>),
    TwoArg(fn(&Expr, &Expr) -> Result<Expr, String>),
    Variadic(fn(&Vec<Expr>) -> Result<Expr, String>),
}

pub fn get_globals() -> HashMap<String, BuiltIn> {
    HashMap::from([
        (
            "nil?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Nil => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "pair?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Nil | Expr::Pair(..) => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "number?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Num(..) => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "boolean?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Boolean(..) => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "string?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::String(..) => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "function?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Lambda(..) => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "symbol?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Keyword(..) => Ok(Expr::Boolean(true)),
                _ => Ok(Expr::Boolean(false)),
            }),
        ),
        (
            "+".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l + r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "-".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l - r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "*".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l * r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            ">".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Boolean(l > r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "<".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Boolean(l < r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "/".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l / r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "%".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l % r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "^".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l.powf(*r))),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "=".to_string(),
            BuiltIn::TwoArg(|l, r| Ok(Expr::Boolean(l == r))),
        ),
        (
            "not".to_string(),
            BuiltIn::OneArg(|arg| match arg {
                Expr::Boolean(arg) => Ok(Expr::Boolean(!arg)),
                _ => Err(format!("Expected boolean, found: {:?}", arg)),
            }),
        ),
        (
            "cons".to_string(),
            BuiltIn::TwoArg(|l, r| Ok(Expr::Pair(Box::new(l.clone()), Box::new(r.clone())))),
        ),
        (
            "car".to_string(),
            BuiltIn::OneArg(|pair| match pair {
                Expr::Pair(box l, ..) => Ok(l.clone()),
                _ => Err(format!("car expected pair, found: {:?}", pair)),
            }),
        ),
        (
            "cdr".to_string(),
            BuiltIn::OneArg(|pair| match pair {
                Expr::Pair(.., box r) => Ok(r.clone()),
                _ => Err(format!("cdr expected pair, found: {:?}", pair)),
            }),
        ),
        (
            "str-append".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::String(l), Expr::String(r)) => Ok(Expr::String(l.clone() + r)),
                _ => Err(format!("Expected strings, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "to-string".to_string(),
            BuiltIn::OneArg(|expr| Ok(Expr::String(expr.to_string()))),
        ),
    ])
}

pub fn make_pairs_from_vec(exprs: Vec<Expr>) -> Expr {
    match exprs.split_first() {
        Some((head, tail)) => Expr::Pair(
            Box::new(head.clone()),
            Box::new(make_pair_from_vec(tail.to_vec())),
        ),
        None => Expr::Nil,
    }
}

pub fn collect_kws_from_expr(expr: &Expr) -> Result<Vec<String>, String> {
    match expr {
        Expr::Pair(box Expr::Keyword(kw), box rest) => collect_kws_from_expr(rest).map(|mut x| {
            x.insert(0, kw.clone());
            x
        }),
        Expr::Nil => Ok(vec![]),
        _ => Err(format!("Invalid keyword list: {:?}", expr)),
    }
}

pub fn collect_exprs_from_body(expr: &Expr) -> Result<Vec<Expr>, String> {
    match expr {
        Expr::Nil => Ok(vec![]),
        Expr::Pair(box expr, box Expr::Nil) => Ok(vec![expr.to_owned()]),
        Expr::Pair(box expr, next @ box Expr::Pair(..)) => {
            collect_exprs_from_body(next).map(|mut x| {
                x.insert(0, expr.to_owned());
                x
            })
        }
        otherwise => Err(format!(
            "tried to collect exprs from body on: {}",
            otherwise
        )),
    }
}
fn make_lambda(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    let (pairs, unextracted_body) = match expr {
        Expr::Pair(pairs @ box Expr::Nil, body @ box Expr::Pair(..)) => (pairs, body),
        Expr::Pair(pairs @ box Expr::Pair(_, _), body @ box Expr::Pair(..)) => (pairs, body),
        otherwise => return Err(format!("Invalid lambda expression: {:?}", otherwise)),
    };

    let body = collect_exprs_from_body(unextracted_body)?;

    // all kws (including rest)

    let all_kws = collect_kws_from_expr(pairs.borrow())?;

    let dot_kw = all_kws
        .iter()
        .enumerate()
        .find(|(_, kw)| *kw == ".")
        .map(|(index, _)| index);

    if let Some(dot_index) = dot_kw {
        // only valid if it's the second to last argument
        if dot_index + 2 != all_kws.len() {
            return Err(format!(
                "rest-dot can only occur as second-to-last argument, but found: {:?}",
                all_kws
            ));
        }
    };

    let rest_arg = dot_kw.and_then(|index| all_kws.get(index + 1));
    let (kws, _) = all_kws.split_at(dot_kw.unwrap_or(all_kws.len()));

    let mut new_body_chunk = Chunk { code: vec![] };

    // find closing over variables
    compile_many_exprs(body.clone(), &mut new_body_chunk, globals)?;

    chunk
        .code
        .push(VMInstruction::Constant(Expr::LambdaDefinition(
            new_body_chunk,
            rest_arg.cloned(),
            kws.to_vec(),
        )));
    chunk.code.push(VMInstruction::MakeLambda);
    Ok(())
}

fn make_define(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    let kw = match expr {
        Expr::Pair(
            box Expr::Pair(box Expr::Keyword(fn_name), fn_args),
            body @ box Expr::Pair(..),
        ) => {
            // this is a lambda definition
            // kws should contain a fn name and then its args
            make_lambda(&Expr::Pair(fn_args.clone(), body.clone()), chunk, globals)?;
            Ok(fn_name.clone())
        }
        Expr::Pair(box Expr::Keyword(kw), box Expr::Pair(box definee, box Expr::Nil)) => {
            compile_internal(definee, chunk, globals)?;
            Ok(kw.clone())
        }
        otherwise => Err(format!(
            "definition, expected kw and expr but found: {:?}",
            otherwise
        )),
    }?;
    chunk.code.push(VMInstruction::Define(kw.clone()));
    chunk.code.push(VMInstruction::Constant(Expr::Nil));
    Ok(())
}

fn make_if(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    let (pred, consequent, alternate) = match expr {
        Expr::Pair(
            box pred,
            box Expr::Pair(box consequent, box Expr::Pair(box alternate, box Expr::Nil)),
        ) => (pred, consequent, alternate),
        otherwise => {
            return Err(format!(
                "if, expected pred, cons, alt but found: {:?}",
                otherwise
            ))
        }
    };

    let mut pred_chunk = Chunk { code: vec![] };
    compile_internal(pred, &mut pred_chunk, globals)?;

    let mut alt_chunk = Chunk { code: vec![] };
    compile_internal(alternate, &mut alt_chunk, globals)?;

    let mut cons_chunk = Chunk { code: vec![] };
    compile_internal(consequent, &mut cons_chunk, globals)?;

    let end_ip = cons_chunk.code.len();

    let cons_ip = 1 + 1 + alt_chunk.code.len();

    chunk.code.extend_from_slice(&pred_chunk.code);

    chunk.code.push(VMInstruction::CondJumpPop(cons_ip));

    chunk.code.extend_from_slice(&alt_chunk.code);

    chunk
        .code
        .push(VMInstruction::Constant(Expr::Boolean(true)));
    chunk.code.push(VMInstruction::CondJumpPop(end_ip));
    chunk.code.extend_from_slice(&cons_chunk.code);
    Ok(())
}

fn make_and(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    let (l, r) = match expr {
        Expr::Pair(box l, box Expr::Pair(box r, box Expr::Nil)) => (l, r),
        otherwise => return Err(format!("and, expected two args but found: {:?}", otherwise)),
    };
    // l + popjmp(r) + jmp(return) + r + return
    let mut r_chunk = Chunk { code: vec![] };
    compile_internal(r, &mut r_chunk, globals)?;
    compile_internal(l, chunk, globals)?;
    chunk.code.push(VMInstruction::CondJump(2));
    chunk
        .code
        .push(VMInstruction::Constant(Expr::Boolean(true)));
    chunk
        .code
        .push(VMInstruction::CondJumpPop(1 + r_chunk.code.len()));
    chunk.code.push(VMInstruction::PopStack);
    chunk.code.extend_from_slice(&r_chunk.code);
    Ok(())
}

fn make_or(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    let (l, r) = match expr {
        Expr::Pair(box l, box Expr::Pair(box r, box Expr::Nil)) => (l, r),
        otherwise => return Err(format!("or, expected two args but found: {:?}", otherwise)),
    };
    let mut r_chunk = Chunk { code: vec![] };
    compile_internal(r, &mut r_chunk, globals)?;
    compile_internal(l, chunk, globals)?;
    chunk
        .code
        .push(VMInstruction::CondJump(1 + r_chunk.code.len()));
    chunk.code.push(VMInstruction::PopStack);
    chunk.code.extend_from_slice(&r_chunk.code);
    Ok(())
}

pub type MacroFn = Arc<dyn Fn(&Vec<Expr>) -> Result<Expr, String>>;

pub fn compile_internal(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    match &expr {
        expr @ Expr::LambdaDefinition(..) | expr @ Expr::Lambda(..) => {
            panic!("Cannot compile a {:?}", expr)
        }
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "lambda" => {
            make_lambda(r, chunk, globals)?;
        }
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "define" => {
            make_define(r, chunk, globals)?;
        }
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "if" => {
            make_if(r, chunk, globals)?;
        }
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "and" => {
            make_and(r, chunk, globals)?;
        }
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "or" => {
            make_or(r, chunk, globals)?;
        }
        Expr::Pair(box Expr::Keyword(kw), box Expr::Pair(box displayee, box Expr::Nil))
            if kw == "display" =>
        {
            compile_internal(&displayee, chunk, globals)?;
            chunk.code.push(VMInstruction::Display);
        }
        Expr::Pair(box Expr::Keyword(kw), box otherwise) if kw == "display" => {
            return Err(format!(
                "Expected one argument for display, but found {}",
                otherwise
            ))
        }
        Expr::Pair(box l, box r) => {
            let exprs = collect_exprs_from_body(r)?;
            if let Expr::Keyword(l) = l {
                let global_arity = match globals.get(l) {
                    Some(BuiltIn::OneArg(..)) => Some(1),
                    Some(BuiltIn::TwoArg(..)) => Some(2),
                    _ => None,
                };
                if global_arity.is_some_and(|arity| arity != exprs.len()) {
                    return Err(format!(
                        "Expected {} arguments for {}, but found {}",
                        global_arity.unwrap(),
                        l,
                        exprs.len(),
                    ));
                }
            }
            compile_internal(l, chunk, globals)?;
            for expr in exprs.iter() {
                compile_internal(expr, chunk, globals)?;
            }
            chunk.code.push(VMInstruction::Call(exprs.len()));
        }
        Expr::Num(nr) => {
            chunk.code.push(VMInstruction::Constant(Expr::Num(*nr)));
        }
        Expr::String(str) => {
            chunk
                .code
                .push(VMInstruction::Constant(Expr::String(str.clone())));
        }
        Expr::Boolean(bool) => {
            chunk
                .code
                .push(VMInstruction::Constant(Expr::Boolean(*bool)));
        }
        Expr::Keyword(kw) => {
            chunk.code.push(VMInstruction::Lookup(kw.clone()));
        }
        Expr::Quote(box expr) => {
            chunk.code.push(VMInstruction::Constant(expr.clone()));
        }
        Expr::Nil => {
            chunk.code.push(VMInstruction::Constant(Expr::Nil));
        }
    };
    Ok(())
}

pub fn compile_many_exprs(
    exprs: Vec<Expr>,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    return exprs.iter().enumerate().try_fold((), |_, (i, expr)| {
        match compile_internal(expr, chunk, globals) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };
        if i == exprs.len() - 1 {
            chunk.code.push(VMInstruction::Return);
            Ok(())
        } else {
            chunk.code.push(VMInstruction::PopStack);
            Ok(())
        }
    });
}

#[test]
fn test_simple_add_compilation() {
    let mut initial_chunk = Chunk { code: vec![] };

    match compile_internal(
        &crate::parse::make_pair_from_vec(vec![
            Expr::Keyword("+".to_string()),
            Expr::Num(1.0),
            Expr::Num(2.0),
        ]),
        &mut initial_chunk,
        &get_globals(),
    ) {
        Ok(_) => {}
        Err(e) => panic!("Error {:?}", e),
    };
    assert_eq!(
        initial_chunk,
        Chunk {
            code: vec![
                VMInstruction::Lookup("+".to_string()),
                VMInstruction::Constant(Expr::Num(1.0)),
                VMInstruction::Constant(Expr::Num(2.0)),
                VMInstruction::Call(2),
            ],
        }
    )
}

#[test]
fn losta_compile() {
    fn parse_and_compile(input: &str) -> Vec<VMInstruction> {
        let expr = crate::parse::parse(input).unwrap().first().unwrap().clone();
        let mut chunk = Chunk { code: vec![] };
        match compile_internal(&expr, &mut chunk, &get_globals()) {
            Ok(..) => chunk.code,
            Err(e) => panic!("Error when compiling {:?}: {:?}", input, e),
        }
    }

    assert_eq!(
        parse_and_compile("(+ 1 2)"),
        vec![
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(Expr::Num(1.0)),
            VMInstruction::Constant(Expr::Num(2.0)),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(
        crate::vm::prepare_vm("(+ 1 2 3)".to_string(), None).map(|x| x.0),
        Err("Expected 2 arguments for +, but found 3".to_string())
    );

    assert_eq!(
        parse_and_compile("((get add) 1 2 3)"),
        vec![
            VMInstruction::Lookup("get".to_string()),
            VMInstruction::Lookup("add".to_string()),
            VMInstruction::Call(1),
            VMInstruction::Constant(Expr::Num(1.0)),
            VMInstruction::Constant(Expr::Num(2.0)),
            VMInstruction::Constant(Expr::Num(3.0)),
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
            VMInstruction::Constant(Expr::Num(1.0)),
            VMInstruction::Constant(Expr::Num(2.0)),
            VMInstruction::Call(2),
            VMInstruction::Constant(Expr::Num(3.0)),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(
        parse_and_compile("()"),
        vec![VMInstruction::Constant(Expr::Nil)]
    );

    assert_eq!(
        parse_and_compile("(f)"),
        vec![
            VMInstruction::Lookup("f".to_string()),
            VMInstruction::Call(0)
        ]
    );

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        vec![
            VMInstruction::Constant(Expr::LambdaDefinition(
                Chunk {
                    code: vec![
                        VMInstruction::Constant(Expr::Num(1.0)),
                        VMInstruction::Return
                    ],
                },
                None,
                vec![],
            )),
            VMInstruction::MakeLambda,
            VMInstruction::Call(0)
        ]
    );
    assert_eq!(
        parse_and_compile("(define a 1)"),
        vec![
            VMInstruction::Constant(Expr::Num(1.0)),
            VMInstruction::Define("a".to_string()),
            VMInstruction::Constant(Expr::Nil),
        ]
    );
}

#[test]
fn lambda_compile_test() {
    fn parse_and_compile(input: &str) -> Chunk {
        let expr = crate::parse::parse(input).unwrap().first().unwrap().clone();
        let mut chunk = Chunk { code: vec![] };
        match compile_internal(&expr, &mut chunk, &get_globals()) {
            Ok(()) => chunk,
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    assert_eq!(
        parse_and_compile("(if 1 2 3)").code,
        vec![
            VMInstruction::Constant(Expr::Num(1.0)),
            VMInstruction::CondJumpPop(3),
            VMInstruction::Constant(Expr::Num(3.0)),
            VMInstruction::Constant(Expr::Boolean(true)),
            VMInstruction::CondJumpPop(1),
            VMInstruction::Constant(Expr::Num(2.0)),
        ]
    );
    assert_eq!(
        parse_and_compile("(lambda () 1)"),
        Chunk {
            code: vec![
                VMInstruction::Constant(Expr::LambdaDefinition(
                    Chunk {
                        code: vec![
                            VMInstruction::Constant(Expr::Num(1.0)),
                            VMInstruction::Return
                        ],
                    },
                    None,
                    vec![],
                )),
                VMInstruction::MakeLambda
            ],
        }
    );

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        Chunk {
            code: vec![
                VMInstruction::Constant(Expr::LambdaDefinition(
                    Chunk {
                        code: vec![
                            VMInstruction::Constant(Expr::Num(1.0)),
                            VMInstruction::Return
                        ],
                    },
                    None,
                    vec![],
                )),
                VMInstruction::MakeLambda,
                VMInstruction::Call(0)
            ],
        }
    );
}
