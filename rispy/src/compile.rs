use std::{borrow::Borrow, collections::HashMap};

use crate::{
    expr::Expr,
    vm::{Chunk, VMInstruction},
};

pub enum BuiltIn {
    OneArg(fn(&Expr) -> Result<Expr, String>),
    TwoArg(fn(&Expr, &Expr) -> Result<Expr, String>),
}

pub fn builtin_arity(builtin: &BuiltIn) -> usize {
    match builtin {
        BuiltIn::OneArg(..) => 1,
        BuiltIn::TwoArg(..) => 2,
    }
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
            "+".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::Num(l + r)),
                _ => Err(format!("Expected numbers, found: {:?} and {:?}", l, r)),
            }),
        ),
        (
            "=".to_string(),
            BuiltIn::TwoArg(|l, r| Ok(Expr::Boolean(l == r))),
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
    ])
}

pub fn compile(expr: &Expr, chunk: &mut Chunk) -> Result<(), String> {
    let globals = get_globals();
    compile_internal(expr, chunk, &globals)
}

fn collect_kws_from_expr(expr: &Expr) -> Result<Vec<String>, String> {
    match expr {
        Expr::Pair(box Expr::Keyword(kw), box rest) => collect_kws_from_expr(rest).map(|mut x| {
            x.insert(0, kw.clone());
            x
        }),
        Expr::Nil => Ok(vec![]),
        _ => Err(format!("Invalid keyword list: {:?}", expr)),
    }
}

fn collect_exprs_from_body(expr: &Expr) -> Result<Vec<Expr>, String> {
    match expr {
        Expr::Pair(box expr, box Expr::Nil) => Ok(vec![expr.to_owned()]),
        Expr::Pair(box expr, next @ box Expr::Pair(..)) => {
            collect_exprs_from_body(next).map(|mut x| {
                x.insert(0, expr.to_owned());
                x
            })
        }
        _ => Ok(vec![]),
    }
}
fn make_lambda(expr: &Expr, chunk: &mut Chunk) -> Result<Chunk, String> {
    let (pairs, unextracted_body) = match expr {
        Expr::Pair(pairs @ box Expr::Nil, body @ box Expr::Pair(..)) => (pairs, body),
        Expr::Pair(pairs @ box Expr::Pair(_, _), body @ box Expr::Pair(..)) => (pairs, body),
        otherwise => return Err(format!("Invalid lambda expression: {:?}", otherwise)),
    };

    let body = collect_exprs_from_body(unextracted_body)?;

    let kws = collect_kws_from_expr(pairs.borrow())?;

    let mut new_body_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    // find closing over variables
    compile_many_exprs(body.clone(), &mut new_body_chunk)?;

    chunk
        .constants
        .push(Expr::LambdaDefinition(new_body_chunk, kws));
    chunk
        .code
        .push(VMInstruction::Constant(chunk.constants.len() - 1));
    chunk.code.push(VMInstruction::MakeLambda);
    Ok(chunk.clone())
}

fn make_define(expr: &Expr, chunk: &mut Chunk) -> Result<(), String> {
    let (kw, definee) = match expr {
        Expr::Pair(box Expr::Keyword(kw), box Expr::Pair(box definee, box Expr::Nil)) => {
            (kw, definee)
        }
        otherwise => {
            return Err(format!(
                "definition, expected kw and expr but found: {:?}",
                otherwise
            ))
        }
    };
    match compile(definee, chunk) {
        Ok(_) => {}
        err @ Err(_) => return err,
    };
    chunk.code.push(VMInstruction::Define(kw.to_string()));
    chunk.constants.push(Expr::Nil);
    chunk
        .code
        .push(VMInstruction::Constant(chunk.constants.len() - 1));
    Ok(())
}

fn make_if(expr: &Expr, chunk: &mut Chunk) -> Result<Chunk, String> {
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
    let mut cons_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };
    compile(consequent, &mut cons_chunk)?;
    compile(pred, chunk)?;
    chunk.code.push(VMInstruction::If(
        cons_chunk.code.len() + 1,
        // one extra return for consequent
    ));
    compile(consequent, chunk)?;
    chunk.code.push(VMInstruction::Return);
    compile(alternate, chunk)?;
    Ok(chunk.clone())
}

pub fn compile_internal(
    expr: &Expr,
    chunk: &mut Chunk,
    globals: &HashMap<String, BuiltIn>,
) -> Result<(), String> {
    match &expr {
        Expr::LambdaDefinition(..) => {
            todo!("Cannot compile a lambda definition (it's already compiled right?)")
        }
        &Expr::Pair(box Expr::Keyword(kw), box r) if kw == "lambda" => {
            make_lambda(r, chunk)?;
        }
        &Expr::Pair(box Expr::Keyword(kw), box r) if kw == "define" => {
            make_define(r, chunk)?;
        }
        &Expr::Pair(box Expr::Keyword(kw), box r) if kw == "if" => {
            make_if(r, chunk)?;
        }
        &Expr::Pair(box Expr::Keyword(kw), box r) if let Some(builtin) = globals.get(kw) => {
            let exprs = collect_exprs_from_body(r)?;
            let arity = builtin_arity(builtin);
            if exprs.len() != arity {
                return Err(format!(
                    "Expected {} arguments for {}, but found {}",
                    arity,
                    kw,
                    exprs.len()
                ));
            }
            for expr in exprs {
                compile_internal(&expr, chunk, globals)?;
            }
            chunk.code.push(VMInstruction::BuiltIn(kw.clone()));
        }
        Expr::Pair(box l, box r) => {
            let exprs = collect_exprs_from_body(r)?;
            compile_internal(l, chunk, globals)?;
            for expr in exprs.iter() {
                compile_internal(expr, chunk, globals)?;
            }
            chunk.code.push(VMInstruction::Call(exprs.len()));
        }
        Expr::Num(nr) => {
            chunk.constants.push(Expr::Num(*nr));
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
        Expr::Boolean(bool) => {
            chunk.constants.push(Expr::Boolean(*bool));
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
        Expr::Keyword(kw) => {
            chunk.code.push(VMInstruction::Lookup(kw.clone()));
        }
        Expr::Lambda(..) => panic!("Cannot compile a Lambda"),
        Expr::Quote(box expr) => {
            chunk.constants.push(expr.clone());
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
        Expr::Nil => {
            chunk.constants.push(Expr::Nil);
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
    };
    Ok(())
}

pub fn compile_many_exprs(exprs: Vec<Expr>, chunk: &mut Chunk) -> Result<(), String> {
    return exprs.iter().enumerate().try_fold((), |_, (i, expr)| {
        match compile(expr, chunk) {
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
    let mut initial_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    match compile(
        &crate::parse::make_pair_from_vec(vec![
            Expr::Keyword("+".to_string()),
            Expr::Num(1.0),
            Expr::Num(2.0),
        ]),
        &mut initial_chunk,
    ) {
        Ok(_) => {}
        Err(e) => panic!("Error {:?}", e),
    };
    assert_eq!(
        initial_chunk,
        Chunk {
            code: vec![
                VMInstruction::Constant(0),
                VMInstruction::Constant(1),
                VMInstruction::BuiltIn("+".to_string()),
            ],
            constants: vec![Expr::Num(1.0), Expr::Num(2.0)],
        }
    )
}

#[test]
fn losta_compile() {
    fn parse_and_compile(input: &str) -> Vec<VMInstruction> {
        let expr = crate::parse::parse(input).unwrap().first().unwrap().clone();
        let mut chunk = Chunk {
            code: vec![],
            constants: vec![],
        };
        match compile(&expr, &mut chunk) {
            Ok(..) => chunk.code,
            Err(e) => panic!("Error when compiling {:?}: {:?}", input, e),
        }
    }

    assert_eq!(
        parse_and_compile("(+ 1 2)"),
        vec![
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::BuiltIn("+".to_string()),
        ]
    );
    assert_eq!(
        crate::vm::prepare_vm("(+ 1 2 3)".to_string()),
        Err("Expected 2 arguments for +, but found 3".to_string())
    );

    assert_eq!(
        parse_and_compile("((get add) 1 2 3)"),
        vec![
            VMInstruction::Lookup("get".to_string()),
            VMInstruction::Lookup("add".to_string()),
            VMInstruction::Call(1),
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::Constant(2),
            VMInstruction::Call(3),
        ]
    );

    assert_eq!(
        parse_and_compile("((get add) (+ 1 2) 3)"),
        vec![
            VMInstruction::Lookup("get".to_string()),
            VMInstruction::Lookup("add".to_string()),
            VMInstruction::Call(1),
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::BuiltIn("+".to_string()),
            VMInstruction::Constant(2),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(parse_and_compile("()"), vec![VMInstruction::Constant(0)]);

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
            VMInstruction::Constant(0),
            VMInstruction::MakeLambda,
            VMInstruction::Call(0)
        ]
    );
    assert_eq!(
        parse_and_compile("(define a 1)"),
        vec![
            VMInstruction::Constant(0),
            VMInstruction::Define("a".to_string()),
            VMInstruction::Constant(1),
        ]
    );
}

#[test]
fn lambda_compile_test() {
    fn parse_and_compile(input: &str) -> Chunk {
        let expr = crate::parse::parse(input).unwrap().first().unwrap().clone();
        let mut chunk = Chunk {
            code: vec![],
            constants: vec![],
        };
        match compile(&expr, &mut chunk) {
            Ok(()) => chunk,
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    assert_eq!(
        parse_and_compile("(lambda () 1)"),
        Chunk {
            code: vec![VMInstruction::Constant(0), VMInstruction::MakeLambda],
            constants: vec![Expr::LambdaDefinition(
                Chunk {
                    code: vec![VMInstruction::Constant(0), VMInstruction::Return],
                    constants: vec![Expr::Num(1.0)]
                },
                vec![],
            ),]
        }
    );

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        Chunk {
            code: vec![
                VMInstruction::Constant(0),
                VMInstruction::MakeLambda,
                VMInstruction::Call(0)
            ],
            constants: vec![Expr::LambdaDefinition(
                Chunk {
                    code: vec![VMInstruction::Constant(0), VMInstruction::Return],
                    constants: vec![Expr::Num(1.0)]
                },
                vec![],
            )]
        }
    );
}
