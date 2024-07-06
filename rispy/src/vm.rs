use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{compile, expr::Expr, parse};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VMInstruction {
    Lookup(String),
    Call(usize),
    Return,
    Constant(usize),
    Add,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Callframe {
    ip: usize,
    chunk: Chunk,
    env: HashMap<String, Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VM {
    callframes: Vec<Callframe>,
    stack: Vec<Expr>,
    globals: HashMap<String, Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub code: Vec<VMInstruction>,
    pub constants: Vec<Expr>,
}

fn run(mut vm: VM) -> Result<VM, String> {
    loop {
        match step(vm) {
            err @ Err(_) => return err,
            Ok(vm) if vm.callframes.len() == 0 => return Ok(vm),
            Ok(new_vm) => {
                vm = new_vm;
            }
        }
    }
}

pub fn step(mut vm: VM) -> Result<VM, String> {
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
        VMInstruction::Lookup(name) => {
            let lookup = match callframe.env.get(name).or(vm.globals.get(name)) {
                Some(instructions) => instructions,
                None => return Err(format!("not found: {name}")),
            };
            vm.stack.push(lookup.clone());
        }
        VMInstruction::Call(arity) => {
            let stack_len = vm.stack.len();
            let first = vm.stack.get(stack_len - arity - 1).cloned();

            let new_callframe = match first {
                Some(Expr::BuiltIn(instructions)) => Callframe {
                    ip: 0,
                    chunk: Chunk {
                        code: instructions.clone(),
                        constants: vec![],
                    },
                    env: HashMap::new(),
                },
                Some(Expr::Lambda(chunk, vars)) => {
                    let x = vm
                        .stack
                        .drain(stack_len - arity..stack_len)
                        .collect::<Vec<Expr>>();
                    let env = vars
                        .iter()
                        .map(|x| x.clone())
                        .zip(x.clone())
                        .collect::<HashMap<String, Expr>>();
                    Callframe {
                        ip: 0,
                        chunk: chunk.to_owned(),
                        env,
                    }
                }
                found => {
                    return Err(format!(
                        "no function to call on stack, found: {:?}, \nstack:{:?}",
                        found, vm.stack
                    ))
                }
            };

            vm.callframes.push(new_callframe);
        }
        VMInstruction::Return => {
            // remove fn from stack?
            let rv = match (vm.stack.pop(), vm.stack.pop()) {
                (Some(rv), Some(Expr::BuiltIn(_))) => rv,
                (Some(rv), Some(Expr::Lambda(_, _))) => rv,
                (Some(_), Some(not_fn)) => {
                    return Err(format!(
                        "expected fn on stack after returning, but found: {:?}\nvm: {:?}",
                        not_fn, vm
                    ))
                }
                _ => return Err(format!("too few args for return on stack: {:?}", vm)),
            };
            vm.stack.push(rv);

            match callframes.pop() {
                Some(_) if callframes.len() == 0 => return Ok(vm.clone()),
                Some(_) => {}
                _ => {
                    return Err("no callframes".to_string());
                }
            }
        }
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
                _ => return Err(format!("too few args for add on stack {:?}", vm.stack)),
            };

            match (arg1, arg2) {
                (Expr::Num(arg1), Expr::Num(arg2)) => vm.stack.push(Expr::Num(arg1 + arg2)),
                _ => return Err("addition requires two numbers".to_string()),
            }
        }
    }
    return Ok(vm.clone());
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

    let callframe = Callframe {
        ip: 0,
        chunk,

        env: HashMap::new(),
    };

    let mut vm = get_initial_vm_and_chunk();
    vm.callframes.push(callframe);

    let result = run(vm);

    debug_assert_eq!(result.map(|x| x.stack), Ok(vec![Expr::Num(3.0)],))
}

fn get_initial_vm_and_chunk() -> VM {
    let globals: HashMap<String, Expr> = HashMap::from([
        (
            "+".to_string(),
            Expr::BuiltIn(vec![VMInstruction::Add, VMInstruction::Return]),
        ),
        (
            "lambda".to_string(),
            Expr::BuiltIn(vec![VMInstruction::Add, VMInstruction::Return]),
        ),
    ]);

    VM {
        callframes: vec![],
        stack: vec![Expr::BuiltIn(vec![])],
        globals,
    }
}

pub fn prepare_vm(input: String) -> Result<VM, String> {
    // parse, compile and run, then check what's left on the stack
    let parsed_expr = parse::parse(&input).and_then(|x| match x.first() {
        Some(res) if x.len() == 1 => Ok(res.clone()),
        _ => Err(format!("expected one expression, got {}", x.len())),
    });

    let expr = match parsed_expr {
        Ok(expr) => expr,
        Err(err) => return Err(err),
    };

    let mut vm = get_initial_vm_and_chunk();

    let mut chunk = Chunk {
        code: vec![],
        constants: vec![],
    };
    match compile::compile(expr, &mut chunk) {
        Ok(_) => {}
        Err(e) => panic!("{:?}", e),
    };
    chunk.code.push(VMInstruction::Return);

    let callframe = Callframe {
        ip: 0,
        chunk,
        env: HashMap::new(),
    };
    vm.callframes.push(callframe);

    return Ok(vm);
}

// just for tests
#[allow(dead_code)]
fn jit_run(input: String) -> Result<Expr, String> {
    let vm = match prepare_vm(input) {
        Ok(vm) => vm,
        Err(err) => return Result::Err(err),
    };

    let interpreted = match run(vm) {
        Ok(e) => e,
        Err(err) => return Result::Err(err),
    };

    match interpreted.stack.get(0) {
        Some(top) if interpreted.stack.len() == 1 => Ok(top.clone()),
        _ => Result::Err(format!(
            "expected one value on the stack, got {:#?}",
            interpreted.stack
        )),
    }
}

fn maybe_log_err<T>(res: Result<T, String>) -> Result<T, String> {
    match res {
        Ok(res) => Ok(res),
        Err(err) => {
            println!("error: {}", err);
            Err(err)
        }
    }
}

#[test]
fn compiled_test() {
    let res = maybe_log_err(jit_run("(+ 1 2)".to_string()));
    assert_eq!(res, Ok(Expr::Num(3.0)));
    let res = maybe_log_err(jit_run("(+ 1 (+ 2 3))".to_string()));
    assert_eq!(res, Ok(Expr::Num(6.0)));
    let res = maybe_log_err(jit_run("(+ (+ 2 3) 1)".to_string()));
    assert_eq!(res, Ok(Expr::Num(6.0)));
    let res = maybe_log_err(jit_run("((lambda () 1))".to_string()));
    assert_eq!(res, Ok(Expr::Num(1.0)));
    let res = maybe_log_err(jit_run("((lambda (a b) (+ a (+ b b))) 1 2)".to_string()));
    assert_eq!(res, Ok(Expr::Num(5.0)));
}
