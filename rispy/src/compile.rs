use crate::{
    expr::Expr,
    vm::{Chunk, VMInstruction},
};

pub fn compile(expr: Expr, chunk: &mut Chunk) {
    match expr {
        Expr::List(exprs) => {
            exprs.iter().for_each(|e| {
                compile(e.clone(), chunk);
            });
            chunk.code.push(VMInstruction::Call(exprs.len() - 1));
        }
        nr @ Expr::Num(_) => {
            chunk.constants.push(nr);
            let index = chunk.constants.len() - 1;
            chunk.code.push(VMInstruction::Constant(index));
        }
        Expr::Keyword(kw) => {
            chunk.code.push(VMInstruction::Lookup(kw));
        }
        Expr::Boolean(_) => todo!("Not yet implemented (boolean)"),
        Expr::Quote(_) => todo!("Not yet implemented (quote)"),
        Expr::VMProc(_) => panic!("Cannot compile a VMProc"),
        Expr::BuiltIn(_) => panic!("Cannot compile a BuiltIn"),
    }
}

#[test]
fn test_simple_add_compilation() {
    let mut initial_chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    compile(
        Expr::List(vec![
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
