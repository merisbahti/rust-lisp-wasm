use crate::{
    compile::{collect_exprs_from_body, MacroFn},
    expr::{Bool, Num},
    macro_expand::macro_expand,
    parse::{make_pair_from_vec, ParseInput},
};
use std::{collections::HashMap, fmt::Display};

use crate::{
    compile::{compile_many_exprs, get_builtins, BuiltIn},
    expr::Expr,
    parse,
};

#[derive(Clone, Debug, PartialEq)]
pub enum VMInstruction {
    Lookup(String),
    MakeLambda,
    Define(String),
    PopStack,
    Apply,
    CondJumpPop(usize),
    CondJump(usize),
    Call(usize),
    Return,
    Display,
    Constant(Expr),
}

impl Display for VMInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMInstruction::Lookup(s) => write!(f, "Lookup({s})"),
            VMInstruction::Define(s) => write!(f, "Define({s})"),
            VMInstruction::CondJumpPop(u) => write!(f, "CondJumpPop({u})"),
            VMInstruction::CondJump(u) => write!(f, "CondJump({u})"),
            VMInstruction::Call(usize) => write!(f, "Call({usize})"),
            VMInstruction::Constant(c) => write!(f, "Constant({c})"),
            VMInstruction::Return => write!(f, "Return"),
            VMInstruction::Display => write!(f, "Display"),
            VMInstruction::PopStack => write!(f, "PopStack"),
            VMInstruction::Apply => write!(f, "Apply"),
            VMInstruction::MakeLambda => write!(f, "MakeLambda"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Env {
    pub map: HashMap<String, Expr>,
    pub parent: Option<String>,
}

pub type HeapAddr = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Callframe {
    pub ip: usize,
    pub chunk: Chunk,
    pub env_addr_mapping: HashMap<String, HeapAddr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VM {
    pub callframes: Vec<Callframe>,
    pub stack: Vec<Expr>,
    pub env: HashMap<HeapAddr, Expr>,
    pub log: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
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
    let env = &mut vm.env;
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
                vm.log.push(format!("{}", top));
                vm.stack.push(Expr::Nil);
            } else {
                return Err("no value to display on stack, compiler bug!".to_string());
            }
        }
        VMInstruction::CondJump(instruction) => {
            let pred = match vm.stack.last() {
                Some(pred) => pred,
                found_vals => {
                    return Err(format!("too few args for if on stack: {:?}", found_vals))
                }
            };
            match pred {
                Expr::Boolean(Bool { value: false, .. })
                | Expr::Nil
                | Expr::Num(Num { value: 0.0, .. }) => (),
                _ => callframe.ip += *instruction,
            }
        }
        VMInstruction::CondJumpPop(instruction) => {
            let pred = match vm.stack.pop() {
                Some(pred) => pred,
                found_vals => {
                    return Err(format!("too few args for if on stack: {:?}", found_vals))
                }
            };
            match pred {
                Expr::Boolean(Bool { value: false, .. })
                | Expr::Nil
                | Expr::Num(Num { value: 0.0, .. }) => (),
                _ => callframe.ip += *instruction,
            }
        }
        VMInstruction::PopStack => {
            vm.stack.pop();
        }
        VMInstruction::Apply => {
            let definition_env = callframe.env.clone();
            let (function_maybe, args_maybe) = match (vm.stack.pop(), vm.stack.pop()) {
                (Some(args @ Expr::Pair(..)), Some(function_maybe)) => (function_maybe, args),
                stuff => {
                    return Err(format!(
                        "apply: expected two args, fn and args, but found: {:?}",
                        stuff
                    ))
                }
            };
            let args = collect_exprs_from_body(&args_maybe)
                .map_err(|_| format!("Expected to apply, but found: {args_maybe}"))?;
            let args_len = args.len();
            let mut instructions: Vec<VMInstruction> = Vec::new();
            instructions.push(VMInstruction::Constant(function_maybe));
            for argument in args {
                instructions.push(VMInstruction::Constant(argument));
            }
            instructions.push(VMInstruction::Call(args_len));
            instructions.push(VMInstruction::Return);
            vm.stack.push(Expr::Lambda(
                Chunk { code: instructions },
                vec![],
                None,
                definition_env,
            ));
        }
        VMInstruction::MakeLambda => {
            // give heap-addresses to closed over variables?
            let definition_env = callframe.env.clone();
            let (instructions, variadic, kws) = match vm.stack.pop() {
                Some(Expr::LambdaDefinition(instructions, variadic, kws)) => {
                    (instructions, variadic, kws)
                }
                stuff => return Err(format!("expected lambda definition, but found: {stuff:#?}",)),
            };
            vm.stack
                .push(Expr::Lambda(instructions, kws, variadic, definition_env));
        }
        VMInstruction::Define(name) => {
            let definee = match vm.stack.pop() {
                Some(to_define) => to_define,
                None => return Err("no value to define".to_string()),
            };
            if let Some(addr_to_define) = callframe.env_addr_mapping.get(name) {
                // should only be defined once?
                if env.contains_key(addr_to_define) {
                    return Err("heap-addr already defined".to_string());
                }
                env.insert(addr_to_define.clone(), definee);
            } else {
                return Err("heap-addr to define didn't exist, compilation error?".to_string());
            }
        }
        VMInstruction::Lookup(name) => {
            let heap_addr = match callframe.env_addr_mapping.get(name) {
                Some(r) => r,
                None => {
                    return Err(format!(
                    "Lookup: {name} didn't exist in the callframe env_addr_mapping, compiler bug?"
                ))
                }
            };

            match env
                .get(heap_addr)
                .cloned()
                .or_else(|| globals.get(name).map(|_| Expr::Keyword(name.clone(), None)))
            {
                Some(expr) => vm.stack.push(expr.clone()),
                None => return Err(format!("{name} at heap-addr: {heap_addr} was not defined.")),
            }
        }
        VMInstruction::Call(arity) => {
            let stack_len = vm.stack.len();
            let first = vm.stack.get(stack_len - arity - 1).cloned();

            match first {
                Some(Expr::Keyword(str, ..)) if let Some(builtin) = globals.get(&str) => {
                    match builtin {
                        BuiltIn::OneArg(func) => {
                            if *arity != 1 {
                                return Err(format!(
                                    "{}, expected one arg but found: {}",
                                    str, arity
                                ));
                            }
                            let top = match vm.stack.pop() {
                                Some(top) => top,
                                None => {
                                    return Err("Expected item on stack, but found none".to_string())
                                }
                            };
                            vm.stack.pop();
                            vm.stack.push(func(&top)?);
                        }
                        BuiltIn::TwoArg(func) => {
                            if *arity != 2 {
                                return Err(format!(
                                    "{}, expected two args but found: {}",
                                    str, arity
                                ));
                            }
                            let (second, first) = match (vm.stack.pop(), vm.stack.pop()) {
                                (Some(first), Some(second)) => (first, second),
                                _ => {
                                    return Err("Expected item on stack, but found none".to_string())
                                }
                            };
                            vm.stack.pop();
                            vm.stack.push(func(&first, &second)?);
                        }
                        BuiltIn::Variadic(func) => {
                            let args = vm
                                .stack
                                .drain(stack_len - arity..stack_len)
                                .collect::<Vec<Expr>>();
                            vm.stack.pop();
                            vm.stack.push(func(&args)?)
                        }
                    }
                }
                Some(Expr::Lambda(chunk, vars, variadic, definition_env)) => {
                    let new_env_name = (envs.len() + 1).to_string();
                    let is_variadic = variadic.is_some();

                    let args = vm
                        .stack
                        .drain(stack_len - arity..stack_len)
                        .collect::<Vec<Expr>>();
                    if is_variadic && args.len() < vars.len() {
                        return Err(format!(
                            "wrong number of args, expected at least {} ({}), got: ({})",
                            vars.len(),
                            vars.join(" "),
                            args.into_iter()
                                .map(|x| format!("{x}"))
                                .collect::<Vec<String>>()
                                .join(" ")
                        ));
                    }

                    if !is_variadic && args.len() != vars.len() {
                        return Err(format!(
                            "wrong number of args, expected {} ({}), got: ({})",
                            vars.len(),
                            vars.join(" "),
                            args.into_iter()
                                .map(|x| format!("{x}"))
                                .collect::<Vec<String>>()
                                .join(" ")
                        ));
                    }

                    let mut map = vars
                        .iter()
                        .cloned()
                        .zip(args.clone())
                        .collect::<HashMap<String, Expr>>();

                    variadic.inspect(|arg_name| {
                        let (_, pairs) = args.split_at(vars.len());
                        map.insert(arg_name.clone(), make_pair_from_vec(pairs.to_vec()));
                    });

                    envs.insert(
                        new_env_name.to_string(),
                        Env {
                            map,
                            parent: Some(definition_env),
                            // parent: None,
                        },
                    );

                    vm.callframes.push(Callframe {
                        ip: 0,
                        chunk: chunk.to_owned(),
                        env: new_env_name.to_string(),
                    });
                }
                found => {
                    return Err(format!(
                        "no function to call on stack, found: {}, \nstack: {}",
                        found.map(|x| format!("{x}")).unwrap_or("None".to_string()),
                        vm.stack
                            .clone()
                            .into_iter()
                            .map(|expr| format!("{expr}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ))
                }
            };
        }
        VMInstruction::Return => {
            // remove fn from stack?
            let rv = match (vm.stack.pop(), vm.stack.pop()) {
                (Some(rv), Some(Expr::Lambda(..))) => rv,
                (Some(rv), None) => rv,
                (Some(a), Some(b)) => {
                    return Err(format!(
                        "expected fn on stack after returning, but found: {a} {b} {}",
                        vm.stack
                            .clone()
                            .into_iter()
                            .map(|x| format!("{x}"))
                            .collect::<Vec<String>>()
                            .join(" ")
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
    }
    Ok(())
}
#[test]
fn test_add() {
    let chunk = Chunk {
        code: vec![
            VMInstruction::Constant(Expr::Keyword("+".to_string(), None)),
            VMInstruction::Constant(Expr::num(1.0)),
            VMInstruction::Constant(Expr::num(2.0)),
            VMInstruction::Call(2),
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

    assert!(run(&mut vm, &get_builtins()).is_ok());

    debug_assert_eq!(vm.stack, vec![Expr::num(3.0)])
}

pub fn get_initial_vm_and_chunk() -> VM {
    VM {
        callframes: vec![],
        stack: vec![],
        env: HashMap::new(),
        log: Vec::new(),
    }
}

pub type Macros = HashMap<String, MacroFn>;
#[derive(Default)]
pub struct CompilerEnv {
    pub env: HashMap<String, Expr>,
    pub macros: Macros,
}

pub fn prepare_vm(
    input: &ParseInput,
    initial_env: Option<CompilerEnv>,
) -> Result<(VM, Macros), String> {
    let compiler_env = initial_env.unwrap_or_default();
    let exprs = match parse::parse(input) {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let mut vm = get_initial_vm_and_chunk();

    let mut chunk = Chunk { code: vec![] };

    let mut macros = compiler_env.macros.clone();
    let macro_expanded = macro_expand(exprs, &mut macros).map_err(|x| x.to_string())?;

    compile_many_exprs(macro_expanded, &mut chunk, &get_builtins())
        .map_err(|err| format!("{err}"))?;

    let callframe = Callframe {
        ip: 0,
        chunk,
        env_addr_mapping: env_addr_mapping,
    };
    vm.callframes.push(callframe);

    Ok((vm, macros))
}

#[allow(dead_code)]
pub fn jit_run_vm(input: &str) -> Result<VM, String> {
    let prelude = get_prelude();
    let (mut vm, _) = match prelude.and_then(|env| {
        prepare_vm(
            &ParseInput {
                source: input,
                file_name: Some("jit_run_vm"),
            },
            Some(env),
        )
    }) {
        Ok(vm) => vm,
        Err(err) => return Result::Err(err),
    };
    run(&mut vm, &get_builtins()).map(|_| vm)
}

// just for tests
#[allow(dead_code)]
pub fn jit_run(input: &str) -> Result<Expr, String> {
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
    let (mut vm, macros) = prepare_vm(
        &ParseInput {
            source: &prelude_string,
            file_name: Some("prelude"),
        },
        None,
    )
    .map_err(|err| format!("Error when compiling prelude: {}", err))?;

    let initial_env_mapping = match vm.callframes.get(0) {
        Some(callframe) => callframe.env_addr_mapping.clone(),
        None => return Err("No initial callframe found when getting prelude".to_string()),
    };

    run(&mut vm, &get_builtins()).map_err(|err| format!("Error when running prelude: {}", err))?;

    if !vm.log.is_empty() {
        return Err(format!(
            "logs were printed happend while evaluating prelude (failed assertions?):\n{}",
            vm.log.join("\n")
        ));
    }
    let mut prelude_env: HashMap<String, Expr> = HashMap::new();
    for (k, v) in initial_env_mapping {
        match vm.env.get(&v) {
            Some(expr) => {
                prelude_env.insert(k, expr.clone());
            }
            None => {
                return Err(format!(
                    "Error when resolving initial env, could not find: {k}"
                ))
            }
        }
    }

    Ok(CompilerEnv {
        env: prelude_env,
        macros,
    })
}

#[test]
fn compiled_test() {
    let res = jit_run("(+ 1 2)");
    assert_eq!(res, Ok(Expr::num(3.0)));
    let res = jit_run("(+ 1 (+ 2 3))");
    assert_eq!(res, Ok(Expr::num(6.0)));
    let res = jit_run("(+ (+ 2 3) 1)");
    assert_eq!(res, Ok(Expr::num(6.0)));
    let res = jit_run("((lambda () 1))");
    assert_eq!(res, Ok(Expr::num(1.0)));
    let res = jit_run("((lambda (a b) (+ a (+ b b))) 1 2)");
    assert_eq!(res, Ok(Expr::num(5.0)));

    assert_eq!(
        jit_run(
            "
(define x 1)
(define y 2)
(+ x y)
        "
        ),
        Ok(Expr::num(3.0))
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
        ),
        Ok(Expr::num(12.0))
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
        ),
        Ok(Expr::num(12.0))
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
        ),
        Err("not found: y".to_string())
    );
    assert_eq!(
        jit_run(
            "
(if 1 2 3)
"
        ),
        Ok(Expr::num(2.0))
    );

    assert_eq!(
        jit_run(
            "
(if (+ 0 0) 2 3)
"
        ),
        Ok(Expr::num(3.0))
    );

    assert_eq!(
        jit_run(
            "
(define f (lambda (a b) 0))
(if (f 3 -3) 2 10)
"
        ),
        Ok(Expr::num(10.0))
    );
    assert_eq!(
        jit_run(
            "
(define f (lambda (x)
  (if x (f (+ x -1)) x
  )))
    
(f 10)"
        ),
        Ok(Expr::num(0.0))
    );
    assert_eq!(jit_run("(= 1 1)"), Ok(Expr::bool(true)));
    assert_eq!(jit_run("(= 1 10)"), Ok(Expr::bool(false)));

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
        ),
        Ok(Expr::num(2.880067194370816e18))
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
        ),
        Ok(Expr::num(89.0))
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
        ),
        Ok(parse::make_pair_from_vec(vec![
            Expr::num(2.0),
            Expr::num(3.0),
            Expr::num(4.0)
        ]))
    );

    assert_eq!(jit_run("(and true true)"), Ok(Expr::bool(true)));
    assert_eq!(jit_run("(and false true)"), Ok(Expr::bool(false)));
    assert_eq!(jit_run("(and true false)"), Ok(Expr::bool(false)));
    assert_eq!(jit_run("(and false false)"), Ok(Expr::bool(false)));
    assert_eq!(jit_run("(or true true)"), Ok(Expr::bool(true)));
    assert_eq!(jit_run("(or false true)"), Ok(Expr::bool(true)));
    assert_eq!(jit_run("(or true false)"), Ok(Expr::bool(true)));
    assert_eq!(jit_run("(or false false)"), Ok(Expr::bool(false)));

    assert_eq!(
        jit_run("(lambda (.) stuff)"),
        Err(
            "jit_run_vm:1:10: rest-dot can only occur as second-to-last argument, but found: (.)"
                .to_string()
        )
    );

    assert_eq!(
        jit_run("(lambda (. more extra) stuff)"),
         Err("jit_run_vm:1:10: rest-dot can only occur as second-to-last argument, but found: (. more extra)".to_string())
    );
    assert_eq!(
        jit_run("(lambda (. more) stuff)"),
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
        jit_run("(lambda (a b . more) stuff)"),
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
        jit_run("((lambda (a b c) (+ (+ a b) c)) 1 2)"),
        Err("wrong number of args, expected 3 (a b c), got: (1 2)".to_string())
    );

    assert_eq!(
        jit_run("((lambda (a b c) (+ (+ a b) c)) 1 2 3 4)"),
        Err("wrong number of args, expected 3 (a b c), got: (1 2 3 4)".to_string())
    );

    assert_eq!(
        jit_run("((lambda (. more) more) 1 2 3 4 5)"),
        Ok(parse::make_pair_from_vec(vec![
            Expr::num(1.0),
            Expr::num(2.0),
            Expr::num(3.0),
            Expr::num(4.0),
            Expr::num(5.0)
        ]))
    );

    assert_eq!(jit_run("(defmacro (m) 10) (m)"), Ok(Expr::num(10.0),));
    assert_eq!(
        jit_run("(defmacro (m a) (cons '+ (cons 1 (cons 2 '())))) (m 2)"),
        Ok(Expr::num(3.0),)
    );
    assert_eq!(
        jit_run("(defmacro (m a) (cons '+ (cons a (cons 2 '())))) (m 1)"),
        Ok(Expr::num(3.0),)
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
                
           ",
        ),
        Ok(Expr::num(11.0))
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
        ),
        parse::parse(&ParseInput {
            source: "(+ 1 (+ 2 0))",
            file_name: None
        })
        .map(|x| match x.first() {
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
        ),
        Ok(Expr::num(3.0),)
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
        ),
        Ok(Expr::num(15.0),)
    );

    assert_eq!(
        jit_run(
            "
            (str-append (str-append \"hello\" \" \") \"world\")
            "
        ),
        Ok(Expr::String("hello world".to_string(), None),)
    );

    assert_eq!(
        jit_run(
            "
            (define add +)
            (add 1 2 3)
"
        ),
        Ok(Expr::num(6.0))
    );

    let example_str = r#"(map (lambda (x) (string? x)) '("hello" (str-append (str-append "hello" " ") "world") 1 2 3))"#;
    assert_eq!(
        jit_run(example_str),
        Ok(make_pair_from_vec(vec![
            Expr::bool(true),
            Expr::bool(false),
            Expr::bool(false),
            Expr::bool(false),
            Expr::bool(false)
        ]))
    );
}
