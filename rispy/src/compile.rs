use crate::{
    expr::Expr,
    vm::{Chunk, VMInstruction},
};

pub fn compile(expr: Expr, chunk: &mut Chunk) {
    compile_internal(expr, chunk, None);
}

pub fn compile_internal(expr: Expr, chunk: &mut Chunk, calling_context: Option<usize>) {
    match expr {
        Expr::Pair(box l, box r) => {
            compile_internal(l, chunk, None);
            compile_internal(r, chunk, calling_context.map(|x| x + 1).or(Some(0)));
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
        Expr::Quote(_) => todo!("Not yet implemented (quote)"),
        Expr::BuiltIn(_) => panic!("Cannot compile a BuiltIn"),
        Expr::Nil => {
            match calling_context {
                Some(calls) => chunk.code.push(VMInstruction::Call(calls)),
                None => return,
            };
        }
    }
}

#[test]
fn test_simple_add_compilation() {
    let mut initial_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    compile(
        crate::parse::make_pair_from_vec(vec![
            Expr::Keyword("+".to_string()),
            Expr::Num(1.0),
            Expr::Num(2.0),
        ]),
        &mut initial_chunk,
    );
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
        compile(expr, &mut chunk);
        chunk.code
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
}
