use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{compile::compile_many_exprs, expr::Expr, parse};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VMInstruction {
    Lookup(String),
    MakeLambda,
    Define(String),
    PopStack,
    Call(usize),
    Return,
    Constant(usize),
    Add,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Env {
    map: HashMap<String, Expr>,
    parent: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Callframe {
    ip: usize,
    chunk: Chunk,
    env: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VM {
    callframes: Vec<Callframe>,
    stack: Vec<Expr>,
    globals: HashMap<String, Expr>,
    envs: HashMap<String, Env>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub code: Vec<VMInstruction>,
    pub constants: Vec<Expr>,
}

fn run(mut vm: VM) -> Result<VM, String> {
    loop {
        match step(&mut vm) {
            Err(err) => return Err(err),
            Ok(()) if vm.callframes.len() == 0 => return Ok(vm),
            Ok(()) => {}
        }
    }
}

pub fn step(vm: &mut VM) -> Result<(), String> {
    // let callframes = &mut vm.callframes;
    // let len = callframes.len();
    let envs = &mut vm.envs;
    let callframe = match vm.callframes.last_mut() {
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
        VMInstruction::PopStack => {
            vm.stack.pop();
        }
        VMInstruction::MakeLambda => {
            let definition_env = callframe.env.clone();
            let (instructions, kws) = match vm.stack.pop() {
                Some(Expr::LambdaDefinition(instructions, kws)) => (instructions, kws),
                stuff => {
                    return Err(format!(
                        "expected lambda definition, but found: {:?}",
                        stuff
                    ))
                }
            };
            vm.stack
                .push(Expr::Lambda(instructions, kws, definition_env));
        }
        VMInstruction::Define(name) => {
            let definee = match vm.stack.pop() {
                Some(to_define) => to_define,
                None => return Err("no value to define".to_string()),
            };

            let env = match envs.get_mut(&callframe.env) {
                Some(env) => env,
                _ => return Err("env missing for callframe".to_string()),
            };
            env.map.insert(name.clone(), definee);
        }
        VMInstruction::Lookup(name) => {
            fn lookup_env(
                name: String,
                env_name: String,
                envs: HashMap<String, Env>,
            ) -> Option<Expr> {
                let env = match envs.get(&env_name) {
                    Some(env) => env,
                    None => panic!("env not found for callframe!!!"),
                };
                let lookup = env.map.get(&name);
                let parent = env.parent.clone();
                match lookup {
                    value @ Some(_) => value.cloned(),
                    _ => match parent {
                        Some(parent_name) => lookup_env(name, parent_name, envs),
                        None => None,
                    },
                }
            }
            let lookup = match lookup_env(name.to_string(), callframe.env.clone(), envs.clone())
                .or(vm.globals.get(name).cloned())
            {
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
                    env: "NONE".to_string(),
                },
                Some(Expr::Lambda(chunk, vars, definition_env)) => {
                    let new_env_name = (envs.len() + 1).to_string();

                    let x = vm
                        .stack
                        .drain(stack_len - arity..stack_len)
                        .collect::<Vec<Expr>>();
                    if x.len() != vars.len() {
                        return Err(format!(
                            "wrong number of args, expected: {:?} ({:?}), got: {:?}",
                            vars.len(),
                            vars,
                            x.len()
                        ));
                    }
                    let map = vars
                        .iter()
                        .map(|x| x.clone())
                        .zip(x.clone())
                        .collect::<HashMap<String, Expr>>();
                    envs.insert(
                        new_env_name.to_string(),
                        Env {
                            map,
                            parent: Some(definition_env),
                            // parent: None,
                        },
                    );
                    Callframe {
                        ip: 0,
                        chunk: chunk.to_owned(),
                        env: new_env_name.to_string(),
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
                (Some(rv), Some(Expr::Lambda(..))) => rv,
                (Some(_), Some(not_fn)) => {
                    return Err(format!(
                        "expected fn on stack after returning, but found: {:?}",
                        not_fn,
                    ))
                }
                _ => return Err(format!("too few args for return on stack",)),
            };
            vm.stack.push(rv);

            match vm.callframes.pop() {
                Some(_) if vm.callframes.len() == 0 => return Ok(()),
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
    return Ok(());
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
        env: "none".to_string(),
    };

    let mut vm = get_initial_vm_and_chunk();
    vm.callframes.push(callframe);

    let result = run(vm);

    debug_assert_eq!(result.map(|x| x.stack), Ok(vec![Expr::Num(3.0)],))
}

fn get_initial_vm_and_chunk() -> VM {
    let globals: HashMap<String, Expr> = HashMap::from([(
        "+".to_string(),
        Expr::BuiltIn(vec![VMInstruction::Add, VMInstruction::Return]),
    )]);

    VM {
        callframes: vec![],
        stack: vec![Expr::BuiltIn(vec![])],
        globals,
        envs: HashMap::from([(
            "initial_env".to_string(),
            Env {
                map: HashMap::new(),
                parent: None,
            },
        )]),
    }
}

pub fn prepare_vm(input: String) -> Result<VM, String> {
    // parse, compile and run, then check what's left on the stack
    let exprs = match parse::parse(&input) {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let mut vm = get_initial_vm_and_chunk();

    let mut chunk = Chunk {
        code: vec![],
        constants: vec![],
    };

    match compile_many_exprs(exprs, &mut chunk) {
        Err(msg) => return Err(msg),
        _ => {}
    };

    let callframe = Callframe {
        ip: 0,
        chunk,
        env: "initial_env".to_string(),
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

    assert_eq!(
        jit_run(
            "
(define x 1)
(define y 2)
(+ x y)
        "
            .to_string()
        ),
        Ok(Expr::Num(3.0))
    );

    assert_eq!(
        jit_run(
            "
(define fn (lambda () 
  (define x 5)
  (define y 7)
  (+ x y)
))
(fn)
        "
            .to_string()
        ),
        Ok(Expr::Num(12.0))
    );

    assert_eq!(
        jit_run(
            "
(define x 5)
(define y 7)
(define fn (lambda () 
  (+ x y)
))
(fn)
        "
            .to_string()
        ),
        Ok(Expr::Num(12.0))
    );

    assert_eq!(
        jit_run(
            "
(define x 1)
(define fnA (lambda () 
    (+ x y)
))
(define fnB (lambda () 
    (define y 5)
    (fnA)
))
(fnB)
        "
            .to_string()
        ),
        Err("not found: y".to_string())
    );
}
