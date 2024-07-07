use std::borrow::Borrow;

use crate::{
    expr::Expr,
    vm::{Chunk, VMInstruction},
};

pub fn compile(expr: Expr, chunk: &mut Chunk) -> Result<Chunk, String> {
    return compile_internal(expr, chunk, None);
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

fn collect_exprs_from_body(expr: &Expr) -> Result<Expr, String> {
    match expr {
        Expr::Pair(box expr, box Expr::Nil) => return Ok(expr.to_owned()),
        _ => Err(format!(
            "Can only compile lambdas with one expr in body, but found: {:?}",
            expr
        )),
    }
}
fn make_lambda(expr: Expr, chunk: &mut Chunk) -> Result<Chunk, String> {
    let (pairs, unextracted_body) = match expr {
        Expr::Pair(pairs @ box Expr::Nil, box body) => (pairs, body),
        Expr::Pair(pairs @ box Expr::Pair(_, _), box body) => (pairs, body),
        otherwise @ _ => return Err(format!("Invalid lambda expression: {:?}", otherwise)),
    };

    let body = match collect_exprs_from_body(&unextracted_body) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let kws = match collect_kws_from_expr(pairs.borrow()) {
        Ok(kws) => kws,
        Err(e) => return Err(format!("{:?}", e)),
    };

    let mut new_body_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    let mut body_compiled = match compile(body.clone(), &mut new_body_chunk) {
        Ok(chunk) => chunk,
        err @ Err(_) => return err,
    };
    body_compiled.code.push(VMInstruction::Return);

    chunk.constants.push(Expr::Lambda(body_compiled, kws));
    chunk
        .code
        .push(VMInstruction::Constant(chunk.constants.len() - 1));
    Ok(chunk.clone())
}

pub fn compile_internal(
    expr: Expr,
    chunk: &mut Chunk,
    calling_context: Option<usize>,
) -> Result<Chunk, String> {
    match expr {
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "lambda".to_string() => {
            return make_lambda(r, chunk);
        }
        Expr::Pair(box Expr::Keyword(kw), box r) if kw == "define".to_string() => {
            chunk.code.push(VMInstruction::Define);
        }
        Expr::Pair(box l, box r) => {
            let _ = compile_internal(l, chunk, None);
            let _ = compile_internal(r, chunk, calling_context.map(|x| x + 1).or(Some(0)));
        }
        nr @ Expr::Num(_) => {
            chunk.constants.push(nr);
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
        bool @ Expr::Boolean(_) => {
            chunk.constants.push(bool);
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
        Expr::Keyword(kw) => {
            chunk.code.push(VMInstruction::Lookup(kw));
        }
        Expr::Lambda(_, _) => panic!("Cannot compile a Lambda"),
        Expr::Quote(_) => todo!("Not yet implemented (quote)"),
        Expr::BuiltIn(_) => panic!("Cannot compile a BuiltIn"),
        Expr::Nil => {
            match calling_context {
                Some(calls) => chunk.code.push(VMInstruction::Call(calls)),
                None => {}
            };
        }
    };
    Ok(chunk.clone())
}

#[test]
fn test_simple_add_compilation() {
    let mut initial_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    match compile(
        crate::parse::make_pair_from_vec(vec![
            Expr::Keyword("+".to_string()),
            Expr::Num(1.0),
            Expr::Num(2.0),
        ]),
        &mut initial_chunk,
    ) {
        Ok(_) => {}
        Err(e) => panic!("Error: {:?}", e),
    };
    assert_eq!(
        initial_chunk,
        Chunk {
            code: vec![
                VMInstruction::Lookup("+".to_string()),
                VMInstruction::Constant(0),
                VMInstruction::Constant(1),
                VMInstruction::Call(2),
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
        match compile(expr, &mut chunk) {
            Ok(e) => return e.code,
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    assert_eq!(
        parse_and_compile("(+ 1 2)"),
        vec![
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(
        parse_and_compile("(+ 1 2 3)"),
        vec![
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::Constant(2),
            VMInstruction::Call(3),
        ]
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
            VMInstruction::Lookup("+".to_string()),
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::Call(2),
            VMInstruction::Constant(2),
            VMInstruction::Call(2),
        ]
    );
    assert_eq!(parse_and_compile("()"), vec![]);
    assert_eq!(
        parse_and_compile("(f)"),
        vec![
            VMInstruction::Lookup("f".to_string()),
            VMInstruction::Call(0)
        ]
    );

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        vec![VMInstruction::Constant(0), VMInstruction::Call(0)]
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
        match compile(expr, &mut chunk) {
            Ok(e) => return e,
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    println!("make lambda");
    assert_eq!(
        parse_and_compile("(lambda () 1)"),
        Chunk {
            code: vec![VMInstruction::Constant(0)],
            constants: vec![Expr::Lambda(
                Chunk {
                    code: vec![VMInstruction::Constant(0), VMInstruction::Return],
                    constants: vec![Expr::Num(1.0)]
                },
                vec![]
            ),]
        }
    );
    println!("make lambda and call");

    assert_eq!(
        parse_and_compile("((lambda () 1))"),
        Chunk {
            code: vec![VMInstruction::Constant(0), VMInstruction::Call(0)],
            constants: vec![Expr::Lambda(
                Chunk {
                    code: vec![VMInstruction::Constant(0), VMInstruction::Return],
                    constants: vec![Expr::Num(1.0)]
                },
                vec![]
            )]
        }
    );
}
