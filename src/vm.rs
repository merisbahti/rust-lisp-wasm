use crate::{compile::MacroFn, macro_expand::macro_expand};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    compile::{compile_many_exprs, get_globals, make_pairs_from_vec, BuiltIn},
    expr::Expr,
    parse,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VMInstruction {
    Lookup(String),
    MakeLambda,
    Define(String),
    PopStack,
    If(usize),
    Call(usize),
    CallBuiltIn(String),
    Return,
    Display,
    Constant(Expr),
    BuiltIn(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Env {
    pub map: HashMap<String, Expr>,
    pub parent: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Callframe {
    pub ip: usize,
    pub chunk: Chunk,
    pub env: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VM {
    pub callframes: Vec<Callframe>,
    pub stack: Vec<Expr>,
    pub envs: HashMap<String, Env>,
    pub log: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub code: Vec<VMInstruction>,
}

pub fn run(vm: &mut VM, globals: &HashMap<String, BuiltIn>) -> Result<(), String> {
    loop {
        match step(vm, globals) {
            Err(err) => return Err(err),
            Ok(()) if vm.callframes.is_empty() => return Ok(()),
            Ok(()) => {}
        }
    }
}

pub fn step(vm: &mut VM, globals: &HashMap<String, BuiltIn>) -> Result<(), String> {
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
        VMInstruction::Display => {
            if let Some(top) = vm.stack.pop() {
                vm.log.push(top.to_string());
                vm.stack.push(Expr::Nil);
            } else {
                return Err("no value to display on stack, compiler bug!".to_string());
            }
        }
        VMInstruction::PopStack => {
            vm.stack.pop();
        }
        VMInstruction::CallBuiltIn(builtin_name) => {
            let builtin = match globals.get(builtin_name) {
                Some(builtin_fn) => builtin_fn,
                None => return Err(format!("no builtin named: {:?}", builtin_name)),
            };
            match builtin {
                BuiltIn::OneArg(func) => {
                    let top = match vm.stack.pop() {
                        Some(top) => top,
                        None => return Err("Expected item on stack, but found none".to_string()),
                    };
                    vm.stack.push(func(&top)?);
                }
                BuiltIn::TwoArg(func) => {
                    let (first, second) = match (vm.stack.pop(), vm.stack.pop()) {
                        (Some(first), Some(second)) => (first, second),
                        _ => return Err("Expected item on stack, but found none".to_string()),
                    };
                    vm.stack.push(func(&first, &second)?);
                }
            }
        }
        VMInstruction::MakeLambda => {
            let definition_env = callframe.env.clone();
            let (instructions, variadic, kws) = match vm.stack.pop() {
                Some(Expr::LambdaDefinition(instructions, variadic, kws)) => {
                    (instructions, variadic, kws)
                }
                stuff => {
                    return Err(format!(
                        "expected lambda definition, but found: {:?}",
                        stuff
                    ))
                }
            };
            vm.stack
                .push(Expr::Lambda(instructions, kws, variadic, definition_env));
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
                envs: &HashMap<String, Env>,
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

            match lookup_env(name.to_string(), callframe.env.clone(), envs) {
                Some(instructions) => {
                    vm.stack.push(instructions.clone());
                    return Ok(());
                }
                None => return Err(format!("not found: {name}",)),
            };
        }
        VMInstruction::Call(arity) => {
            let stack_len = vm.stack.len();
            let first = vm.stack.get(stack_len - arity - 1).cloned();

            let new_callframe = match first {
                Some(Expr::Lambda(chunk, vars, variadic, definition_env)) => {
                    let new_env_name = (envs.len() + 1).to_string();
                    let is_variadic = variadic.is_some();

                    let args = vm
                        .stack
                        .drain(stack_len - arity..stack_len)
                        .collect::<Vec<Expr>>();
                    if is_variadic && args.len() < vars.len() {
                        return Err(format!(
                            "wrong number of args, expected at least {:?} ({:?}), got: {:?}",
                            vars.len(),
                            vars,
                            args.len()
                        ));
                    }

                    if !is_variadic && args.len() != vars.len() {
                        return Err(format!(
                            "wrong number of args, expected: {:?} ({:?}), got: {:?}",
                            vars.len(),
                            vars,
                            args.len()
                        ));
                    }

                    let mut map = vars
                        .iter()
                        .cloned()
                        .zip(args.clone())
                        .collect::<HashMap<String, Expr>>();

                    variadic.inspect(|arg_name| {
                        let (_, pairs) = args.split_at(vars.len());
                        map.insert(arg_name.clone(), make_pairs_from_vec(pairs.to_vec()));
                    });

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
                (Some(rv), Some(Expr::Lambda(..))) => rv,
                (Some(rv), None) => rv,
                (Some(a), Some(b)) => {
                    return Err(format!(
                        "expected fn on stack after returning, but found: {a} {b} {:?}",
                        vm.stack
                    ))
                }
                _ => return Err("too few args for return on stack".to_string()),
            };
            vm.stack.push(rv);

            match vm.callframes.pop() {
                Some(_) if vm.callframes.is_empty() => return Ok(()),
                Some(_) => {}
                _ => {
                    return Err("no callframes".to_string());
                }
            }
        }
        VMInstruction::Constant(expr) => {
            vm.stack.push(expr.clone());
        }
        VMInstruction::If(alt_ip) => {
            let pred = match vm.stack.pop() {
                Some(pred) => pred,
                found_vals => {
                    return Err(format!("too few args for if on stack: {:?}", found_vals))
                }
            };
            match pred {
                Expr::Boolean(false) | Expr::Nil | Expr::Num(0.0) => callframe.ip += *alt_ip,
                _ => {}
            }
        }
        VMInstruction::BuiltIn(str) if let Some(BuiltIn::OneArg(f)) = globals.get(str) => {
            let value = match vm.stack.pop() {
                Some(pred) => pred,
                found_vals => {
                    return Err(format!(
                        "too few args for if on stack for ({:?}): {:?}",
                        str, found_vals
                    ))
                }
            };
            vm.stack.push(f(&value)?);
        }
        VMInstruction::BuiltIn(str) if let Some(BuiltIn::TwoArg(f)) = globals.get(str) => {
            let (top, bot) = match (vm.stack.pop(), vm.stack.pop()) {
                (Some(bot), Some(top)) => (top, bot),
                found_vals => {
                    return Err(format!(
                        "too few args for if on stack for ({:?}): {:?}",
                        str, found_vals
                    ))
                }
            };
            vm.stack.push(f(&top, &bot)?);
        }
        VMInstruction::BuiltIn(name) => {
            return Err(format!("unknown built-in: {name}"));
        }
    }
    Ok(())
}
#[test]
fn test_add() {
    let chunk = Chunk {
        code: vec![
            VMInstruction::Constant(Expr::Num(1.0)),
            VMInstruction::Constant(Expr::Num(2.0)),
            VMInstruction::BuiltIn("+".to_string()),
            VMInstruction::Return,
        ],
    };

    let callframe = Callframe {
        ip: 0,
        chunk,
        env: "none".to_string(),
    };

    let mut vm = get_initial_vm_and_chunk(Env::default());
    vm.callframes.push(callframe);

    assert!(run(&mut vm, &get_globals()).is_ok());

    debug_assert_eq!(vm.stack, vec![Expr::Num(3.0)])
}

pub fn get_initial_vm_and_chunk(initial_env: Env) -> VM {
    VM {
        callframes: vec![],
        stack: vec![],
        envs: HashMap::from([("initial_env".to_string(), initial_env)]),
        log: Vec::new(),
    }
}

pub type Macros = HashMap<String, MacroFn>;
#[derive(Default)]
pub struct CompilerEnv {
    pub env: Env,
    pub macros: Macros,
}

pub fn prepare_vm(input: String, initial_env: Option<CompilerEnv>) -> Result<(VM, Macros), String> {
    let compiler_env = initial_env.unwrap_or_default();
    let exprs = match parse::parse(&input) {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let mut vm = get_initial_vm_and_chunk(compiler_env.env);

    let mut chunk = Chunk { code: vec![] };

    let mut macros = compiler_env.macros.clone();
    let macro_expanded = macro_expand(exprs, &mut macros)?;

    compile_many_exprs(macro_expanded, &mut chunk, &get_globals())?;

    let callframe = Callframe {
        ip: 0,
        chunk,
        env: "initial_env".to_string(),
    };
    vm.callframes.push(callframe);

    Ok((vm, macros))
}

#[allow(dead_code)]
pub fn jit_run_vm(input: String) -> Result<VM, String> {
    let prelude = get_prelude();
    let (mut vm, _) = match prelude.and_then(|env| prepare_vm(input, Some(env))) {
        Ok(vm) => vm,
        Err(err) => return Result::Err(err),
    };
    run(&mut vm, &get_globals()).map(|_| vm)
}

// just for tests
#[allow(dead_code)]
pub fn jit_run(input: String) -> Result<Expr, String> {
    let vm = jit_run_vm(input)?;
    match vm.stack.first() {
        Some(top) if vm.stack.len() == 1 => Ok(top.clone()),
        _ => Result::Err(format!(
            "expected one value on the stack, got {:#?}",
            vm.stack
        )),
    }
}

pub fn get_prelude() -> Result<CompilerEnv, String> {
    let prelude_string = include_str!("../prelude.scm").to_string();
    let (mut vm, macros) = prepare_vm(prelude_string, None)
        .map_err(|err| format!("Error when compiling prelude: {:?}", err))?;
    run(&mut vm, &get_globals()).map_err(|err| format!("Error when running prelude: {:?}", err))?;

    if !vm.log.is_empty() {
        return Err(format!(
            "logs were printed happend while evaluating prelude (failed assertions?): {:?}",
            vm.log
        ));
    }

    match vm.envs.get("initial_env") {
        Some(env) => Ok(CompilerEnv {
            env: env.clone(),
            macros,
        }),
        None => Err("Could not find initial env when compiling prelude.".to_string()),
    }
}

#[test]
fn compiled_test() {
    let res = jit_run("(+ 1 2)".to_string());
    assert_eq!(res, Ok(Expr::Num(3.0)));
    let res = jit_run("(+ 1 (+ 2 3))".to_string());
    assert_eq!(res, Ok(Expr::Num(6.0)));
    let res = jit_run("(+ (+ 2 3) 1)".to_string());
    assert_eq!(res, Ok(Expr::Num(6.0)));
    let res = jit_run("((lambda () 1))".to_string());
    assert_eq!(res, Ok(Expr::Num(1.0)));
    let res = jit_run("((lambda (a b) (+ a (+ b b))) 1 2)".to_string());
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
    assert_eq!(
        jit_run(
            "
(if 1 2 3)
"
            .to_string()
        ),
        Ok(Expr::Num(2.0))
    );

    assert_eq!(
        jit_run(
            "
(if (+ 0 0) 2 3)
"
            .to_string()
        ),
        Ok(Expr::Num(3.0))
    );

    assert_eq!(
        jit_run(
            "
(define f (lambda (a b) 0))
(if (f 3 -3) 2 10)
"
            .to_string()
        ),
        Ok(Expr::Num(10.0))
    );
    assert_eq!(
        jit_run(
            "
(define f (lambda (x)
  (if x (f (+ x -1)) x
  )))
    
(f 10)"
                .to_string()
        ),
        Ok(Expr::Num(0.0))
    );
    assert_eq!(jit_run("(= 1 1)".to_string()), Ok(Expr::Boolean(true)));
    assert_eq!(jit_run("(= 1 10)".to_string()), Ok(Expr::Boolean(false)));

    assert_eq!(
        jit_run(
            "
(define fib (lambda (n) 
  (fib-iter 1 0 n)))
(define fib-iter (lambda (a b count)
(if (= count 0)
  b
  (fib-iter (+ a b) a (+ count -1)))))
(fib 90)"
                .to_string()
        ),
        Ok(Expr::Num(2.880067194370816e18))
    );

    assert_eq!(
        jit_run(
            "
(define fib (lambda (n) 
  (if 
  (= n 0) 1
  (if (= n 1) 1
    (+ (fib (+ n -1)) (fib (+ n -2)))  )
  )))
  
(fib 10)"
                .to_string()
        ),
        Ok(Expr::Num(89.0))
    );

    assert_eq!(
        jit_run(
            "
(define map (lambda (proc items)
  (if 
    (nil? items) 
    '()
    (cons (proc (car items)) (map proc (cdr items))))))
(map (lambda (x) (+ 1 x)) '(1 2 3))"
                .to_string()
        ),
        Ok(parse::make_pair_from_vec(vec![
            Expr::Num(2.0),
            Expr::Num(3.0),
            Expr::Num(4.0)
        ]))
    );

    assert_eq!(
        jit_run("(and true true)".to_string()),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        jit_run("(and false true)".to_string()),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        jit_run("(and true false)".to_string()),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        jit_run("(and false false)".to_string()),
        Ok(Expr::Boolean(false))
    );
    assert_eq!(
        jit_run("(or true true)".to_string()),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        jit_run("(or false true)".to_string()),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        jit_run("(or true false)".to_string()),
        Ok(Expr::Boolean(true))
    );
    assert_eq!(
        jit_run("(or false false)".to_string()),
        Ok(Expr::Boolean(false))
    );

    assert_eq!(
        jit_run("(lambda (.) stuff)".to_string()),
        Err("rest-dot can only occur as second-to-last argument, but found: [\".\"]".to_string())
    );

    assert_eq!(
        jit_run("(lambda (. more extra) stuff)".to_string()),
        Err("rest-dot can only occur as second-to-last argument, but found: [\".\", \"more\", \"extra\"]".to_string())
    );
    assert_eq!(
        jit_run("(lambda (. more) stuff)".to_string()),
        Ok(Expr::Lambda(
            Chunk {
                code: vec![
                    VMInstruction::Lookup("stuff".to_string()),
                    VMInstruction::Return
                ]
            },
            vec![],
            Some("more".to_string()),
            "initial_env".to_string()
        ))
    );

    assert_eq!(
        jit_run("(lambda (a b . more) stuff)".to_string()),
        Ok(Expr::Lambda(
            Chunk {
                code: vec![
                    VMInstruction::Lookup("stuff".to_string()),
                    VMInstruction::Return
                ]
            },
            vec!["a".to_string(), "b".to_string()],
            Some("more".to_string()),
            "initial_env".to_string()
        ))
    );

    assert_eq!(
        jit_run("((lambda (a b c) (+ (+ a b) c)) 1 2)".to_string()),
        Err("wrong number of args, expected: 3 ([\"a\", \"b\", \"c\"]), got: 2".to_string())
    );

    assert_eq!(
        jit_run("((lambda (a b c) (+ (+ a b) c)) 1 2 3 4)".to_string()),
        Err("wrong number of args, expected: 3 ([\"a\", \"b\", \"c\"]), got: 4".to_string())
    );

    assert_eq!(
        jit_run("((lambda (. more) more) 1 2 3 4 5)".to_string()),
        Ok(parse::make_pair_from_vec(vec![
            Expr::Num(1.0),
            Expr::Num(2.0),
            Expr::Num(3.0),
            Expr::Num(4.0),
            Expr::Num(5.0)
        ]))
    );

    assert_eq!(
        jit_run("(defmacro (m) 10) (m)".to_string()),
        Ok(Expr::Num(10.0),)
    );
    assert_eq!(
        jit_run("(defmacro (m a) (cons '+ (cons 1 (cons 2 '())))) (m 2)".to_string()),
        Ok(Expr::Num(3.0),)
    );
    assert_eq!(
        jit_run("(defmacro (m a) (cons '+ (cons a (cons 2 '())))) (m 1)".to_string()),
        Ok(Expr::Num(3.0),)
    );
    assert_eq!(
        jit_run(
            "
                (define (accumulate op initial sequence)
                  (if (nil? sequence)
                    initial
                    (op (car sequence)
                     (accumulate op initial (cdr sequence)))))
                     (accumulate (lambda (curr acc) (+ curr acc))
                     0
                     '(1 2 3 5)
                     
                     )
                
           "
            .to_string(),
        ),
        Ok(Expr::Num(11.0))
    );

    assert_eq!(
        jit_run(
            "
            (define  
                (add . xs) 
                (define (accumulate op initial sequence)
                  (if (nil? sequence)
                    initial
                    (op (car sequence)
                     (accumulate op initial (cdr sequence)))))
                (accumulate 
                  (lambda (curr acc) (cons '+ (cons curr (cons acc '()))))
                  0
                  xs)
            ) 
            (add 1 2)
            "
            .to_string()
        ),
        parse::parse("(+ 1 (+ 2 0))").map(|x| match x.first() {
            Some(x) => x.clone(),
            None => panic!(),
        })
    );
    assert_eq!(
        jit_run(
            "
            (defmacro  
                (add . xs) 
                (define (accumulate op initial sequence)
                  (if (nil? sequence)
                    initial
                    (op (car sequence)
                     (accumulate op initial (cdr sequence)))))
                (accumulate 
                  (lambda (curr acc) (cons '+ (cons curr (cons acc '()))))
                  0
                  xs)
            ) 
            (add 1 2)
            "
            .to_string()
        ),
        Ok(Expr::Num(3.0),)
    );
    assert_eq!(
        jit_run(
            "
            (defmacro  
                (add . xs) 
                (define (accumulate op initial sequence)
                  (if (nil? sequence)
                    initial
                    (op (car sequence)
                     (accumulate op initial (cdr sequence)))))
                (accumulate 
                  (lambda (curr acc) (cons '+ (cons curr (cons acc '()))))
                  0
                  xs)
            ) 
            (add 1 2 3 4 5)
            "
            .to_string()
        ),
        Ok(Expr::Num(15.0),)
    );

    assert_eq!(
        jit_run(
            "
            (str-append (str-append \"hello\" \" \") \"world\")
            "
            .to_string()
        ),
        Ok(Expr::String("hello world".to_string()),)
    );

    let example_str = r#"(map (lambda (x) (string? x)) '("hello" (str-append (str-append "hello" " ") "world") 1 2 3))"#;
    assert_eq!(
        jit_run(example_str.to_string()),
        Ok(make_pairs_from_vec(vec![
            Expr::Boolean(true),
            Expr::Boolean(false),
            Expr::Boolean(false),
            Expr::Boolean(false),
            Expr::Boolean(false)
        ]))
    );
}
