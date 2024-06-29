use crate::expr::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum VMInstruction {
    Lookup(String),
    Call,
    Return,
    Print,
    Constant(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
}

#[derive(Clone, Debug, PartialEq, Copy)]
struct Callframe {
    return_ip: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct VM {
    call_frames: Vec<Callframe>,
    ip: usize,
    stack: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk {
    pub code: Vec<VMInstruction>,
    pub constants: Vec<Expr>,
}

fn run(chunk: &Chunk, mut vm: VM) -> Result<VM, String> {
    loop {
        let instruction = if let Some(instruction) = chunk.code.get(vm.ip) {
            instruction
        } else {
            return Err("End of code reached".to_string());
        };
        vm.ip += 1;
        match instruction {
            VMInstruction::Lookup(kw) => if (kw == "print") {},
            VMInstruction::Call => {
                // check if bot of stack points to a function???
                todo!();
            }
            VMInstruction::Return => {
                let frame = if let Some(frame) = vm.call_frames.pop() {
                    frame
                } else {
                    return Ok(vm);
                };
                vm.ip = frame.return_ip;
            }
            VMInstruction::Print => {
                let argument = if let Some(argument) = vm.stack.pop() {
                    argument
                } else {
                    return Err("too few args for print on stack".to_string());
                };
                println!("{}", argument);
            }
            VMInstruction::Constant(arg) => {
                if let Some(constant) = chunk.constants.get(arg.clone()) {
                    vm.stack.push(constant.clone());
                } else {
                    return Err(format!("constant not found: {arg}"));
                }
            }
            VMInstruction::Add => {
                let (arg1, arg2) =
                    if let (Some(arg1), Some(arg2)) = (vm.stack.pop(), vm.stack.pop()) {
                        (arg1, arg2)
                    } else {
                        return Err("too few args for add on stack".to_string());
                    };
                if let (Expr::Num(arg1), Expr::Num(arg2)) = (arg1, arg2) {
                    vm.stack.push(Expr::Num(arg1 + arg2));
                } else {
                    return Err("addition requires two numbers".to_string());
                }
            }
            VMInstruction::Subtract => todo!(),
            VMInstruction::Multiply => todo!(),
            VMInstruction::Divide => todo!(),
            VMInstruction::Negate => todo!(),
        }
    }
}

fn interpret_chunk(chunk: &Chunk, vm: VM) -> Result<VM, String> {
    return run(chunk, vm);
}

#[test]
fn test_interpreter() {
    let vm = VM {
        call_frames: vec![],
        ip: 0,
        stack: vec![],
    };
    let chunk = Chunk {
        code: vec![VMInstruction::Return],
        constants: vec![],
    };
    assert_eq!(
        interpret_chunk(&chunk, vm),
        Ok(VM {
            call_frames: vec![],
            ip: 1,
            stack: vec![],
        })
    )
}

#[test]
fn test_add() {
    let vm = VM {
        call_frames: vec![],
        ip: 0,
        stack: vec![],
    };
    let chunk = Chunk {
        code: vec![
            VMInstruction::Constant(0),
            VMInstruction::Constant(1),
            VMInstruction::Add,
            VMInstruction::Return,
        ],
        constants: vec![Expr::Num(1.0), Expr::Num(2.0)],
    };
    assert_eq!(
        interpret_chunk(&chunk, vm),
        Ok(VM {
            call_frames: vec![],
            ip: 4,
            stack: vec![Expr::Num(3.0)],
        })
    )
}
