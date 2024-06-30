use std::collections::HashMap;

use crate::{compile, expr::Expr, parse};

#[derive(Clone, Debug, PartialEq)]
pub enum VMInstruction {
    Lookup(String),
    Call,
    Return,
    Constant(usize),
    Add,
}

#[derive(Clone, Debug, PartialEq)]
struct Callframe {
    ip: usize,
    chunk: Chunk,
}

#[derive(Clone, Debug, PartialEq)]
struct VM {
    callframes: Vec<Callframe>,
    stack: Vec<Expr>,
    built_ins: HashMap<String, Vec<VMInstruction>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub code: Vec<VMInstruction>,
    pub constants: Vec<Expr>,
}

fn run(mut vm: VM) -> Result<VM, String> {
    loop {
        let callframes = &mut vm.callframes;
        let len = callframes.len();
        let callframe = match callframes.get_mut(len - 1) {
            Some(x) => x,
            _ => return Err("no callframes".to_string()),
        };
        let chunk = &callframe.chunk;
        let instruction = if let Some(instruction) = chunk.code.get(callframe.ip) {
            instruction
        } else {
            return Err("End of code reached".to_string());
        };
        callframe.ip += 1;
        match instruction {
            VMInstruction::Lookup(_) => todo!("lookup"),
            VMInstruction::Call => {
                // check if bot of stack points to a function???
                let new_callframe = Callframe {
                    ip: 0,
                    chunk: Chunk {
                        code: vec![],
                        constants: vec![],
                    },
                };

                vm.callframes.push(new_callframe);

                todo!("call nyi");
            }
            VMInstruction::Return => match callframes.pop() {
                Some(_) if callframes.len() == 0 => return Ok(vm),
                Some(_) => {}
                _ => {
                    return Err("no callframes".to_string());
                }
            },
            VMInstruction::Constant(arg) => {
                if let Some(constant) = chunk.constants.get(arg.clone()) {
                    vm.stack.push(constant.clone());
                } else {
                    return Err(format!("constant not found: {arg}"));
                }
            }
            VMInstruction::Add => {
                let (arg1, arg2) = match (vm.stack.pop(), vm.stack.pop()) {
                    (Some(arg1), Some(arg2)) => (arg1, arg2),
                    _ => return Err("too few args for add on stack".to_string()),
                };

                match (arg1, arg2) {
                    (Expr::Num(arg1), Expr::Num(arg2)) => vm.stack.push(Expr::Num(arg1 + arg2)),
                    _ => return Err("addition requires two numbers".to_string()),
                }
            }
        }
    }
}

#[test]
fn test_interpreter() {
    let chunk = Chunk {
        code: vec![VMInstruction::Return],
        constants: vec![],
    };
    let vm = VM {
        callframes: vec![Callframe { ip: 0, chunk }],
        stack: vec![],
        built_ins: HashMap::new(),
    };
    assert_eq!(
        run(vm),
        Ok(VM {
            callframes: vec![],
            stack: vec![],
            built_ins: HashMap::new(),
        })
    )
}

#[test]
fn test_add() {
    let chunk = Chunk {
        code: vec![
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::Add,
            VMInstruction::Return,
        ],
        constants: vec![Expr::Num(1.0), Expr::Num(2.0)],
    };

    let callframe = Callframe { ip: 0, chunk };

    let vm = VM {
        callframes: vec![callframe],
        stack: vec![],
        built_ins: HashMap::new(),
    };

    assert_eq!(
        run(vm),
        Ok(VM {
            callframes: vec![],
            stack: vec![Expr::Num(3.0)],
            built_ins: HashMap::new(),
        })
    )
}

fn get_initial_vm_and_chunk() -> (VM, Chunk) {
    let built_ins: HashMap<String, Vec<VMInstruction>> = HashMap::from([(
        "+".to_string(),
        vec![VMInstruction::Add, VMInstruction::Return],
    )]);

    let chunk = Chunk {
        code: vec![],
        constants: vec![],
    };
    let vm = VM {
        callframes: vec![],
        stack: vec![],
        built_ins,
    };
    (vm, chunk)
}

// just for tests
#[allow(dead_code)]
fn jit_run(input: String) -> Result<Expr, String> {
    // parse, compile and run, then check what's left on the stack
    let parsed_expr = parse::parse(&input).and_then(|x| match x.first() {
        Some(res) if x.len() == 1 => Ok(res.clone()),
        _ => Err(format!("expected one expression, got {}", x.len())),
    });

    let expr = match parsed_expr {
        Ok(expr) => expr,
        e @ Err(_) => return e,
    };

    let (mut vm, mut chunk) = get_initial_vm_and_chunk();

    compile::compile(expr, &mut chunk);

    print!("vm: {:?}", vm);
    let callframe = Callframe { ip: 0, chunk };
    vm.callframes.push(callframe);

    let interpreted = match run(vm) {
        Ok(e) => e,
        Err(err) => return Result::Err(err),
    };

    match interpreted.stack.get(0) {
        Some(top) if interpreted.stack.len() == 1 => Ok(top.clone()),
        _ => Result::Err(format!(
            "expected one value on the stack, got {}",
            interpreted.stack.len()
        )),
    }
}

#[test]
fn compiled_test() {
    let res = jit_run("(+ 1 2)".to_string());
    assert_eq!(res, Ok(Expr::Num(3.0)));
}
