use std::{borrow::Borrow, collections::HashMap, fmt::Display, sync::Arc};

use once_cell::sync::Lazy;

use crate::{
    expr::{Bool, Expr, Num},
    parse::SrcLoc,
    vm::{Chunk, VMInstruction},
};

pub enum BuiltIn {
    OneArg(fn(&Expr) -> Result<Expr, String>),
    TwoArg(fn(&Expr, &Expr) -> Result<Expr, String>),
    Variadic(fn(&Vec<Expr>) -> Result<Expr, String>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompileError {
    pub srcloc: Option<SrcLoc>,
    pub message: String,
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message.clone();
        if let Some(srcloc) = self.srcloc.clone() {
            write!(
                f,
                "{}:{}:{}: {message}",
                srcloc.file_name.unwrap_or("unknown".to_string()),
                srcloc.line,
                srcloc.column
            )
        } else {
            write!(f, "unknown: {message}")
        }
    }
}

#[macro_export]
macro_rules! comp_err {
    ($srcloc:expr, $fmt_str:literal) => {
        {
            Result::<(), CompileError>::Err(
                CompileError {
                    message: format!($fmt_str),
                    srcloc: extract_srcloc($srcloc)
                }
            )
        }
    };
    ($srcloc:expr, $fmt_str:literal, $($args:expr),*) => {
        {
            Result::<(), CompileError>::Err(
                CompileError {
                    message: format!($fmt_str, $($args),*),
                    srcloc: extract_srcloc($srcloc)
                }
            )
        }
    };
}

pub fn extract_srcloc(expr: &Expr) -> Option<SrcLoc> {
    match expr {
        Expr::Keyword(_, s) => s,
        Expr::Pair(_, _, s) => s,
        Expr::String(_, s) => s,
        Expr::Quote(_, s) => s,
        Expr::Num(Num { srcloc: s, .. }) => s,
        Expr::Boolean(Bool { srcloc: s, .. }) => s,
        Expr::Lambda(_, _, _, _) => todo!("Not implemented src_loc for this lambda."),
        Expr::LambdaDefinition(_, _, _) => todo!("Not implemented src_loc for this lambda-def."),
        Expr::Nil => &Some(SrcLoc {
            line: 13391339,
            column: 0,
            file_name: None,
        }),
    }
    .clone()
}

type CompileResult = Result<(), CompileError>;

pub static BUILTIN_FNS: Lazy<HashMap<String, BuiltIn>> = Lazy::new(|| {
    let globals = HashMap::from([
        (
            "nil?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Nil => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "pair?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Nil | Expr::Pair(..) => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "number?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Num(..) => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "boolean?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Boolean(..) => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "string?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::String(..) => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "abs".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Num(nr) => Ok(Expr::num(nr.value.abs())),
                other => Err(format!("abs: expected num but found: {other}")),
            }),
        ),
        (
            "function?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Lambda(..) => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "symbol?".to_string(),
            BuiltIn::OneArg(|expr| match expr {
                Expr::Keyword(..) => Ok(Expr::bool(true)),
                _ => Ok(Expr::bool(false)),
            }),
        ),
        (
            "+".to_string(),
            BuiltIn::Variadic(|args| {
                (args.clone())
                    .into_iter()
                    .try_reduce::<Result<Expr, String>>(|acc, curr| {
                        match (acc.clone(), curr.clone()) {
                            (Expr::Num(Num { value: l, .. }), Expr::Num(Num { value: r, .. })) => {
                                Ok(Expr::num(l + r))
                            }
                            _ => Err(format!("Expected numbers, found: {} and {}", acc, curr)),
                        }
                    })
                    .map(|x| x.unwrap_or_else(|| Expr::num(0.0)))
            }),
        ),
        (
            "-".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::num(l.value - r.value)),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            "*".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::num(l.value * r.value)),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            ">".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::bool(l.value > r.value)),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            "<".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::bool(l.value < r.value)),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            "/".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::num(l.value / r.value)),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            "%".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::num(l.value % r.value)),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            "^".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::Num(l), Expr::Num(r)) => Ok(Expr::num(l.value.powf((*r).value))),
                _ => Err(format!("Expected numbers, found: {} and {}", l, r)),
            }),
        ),
        (
            "=".to_string(),
            BuiltIn::TwoArg(|l, r| Ok(Expr::bool(l == r))),
        ),
        (
            "not".to_string(),
            BuiltIn::OneArg(|arg| match arg {
                Expr::Boolean(arg) => Ok(Expr::bool(!arg.value)),
                _ => Err(format!("Expected boolean, found: {}", arg)),
            }),
        ),
        (
            "cons".to_string(),
            BuiltIn::TwoArg(|l, r| Ok(Expr::Pair(Box::new(l.clone()), Box::new(r.clone()), None))),
        ),
        (
            "car".to_string(),
            BuiltIn::OneArg(|pair| match pair {
                Expr::Pair(box l, ..) => Ok(l.clone()),
                _ => Err(format!("car expected pair, found: {}", pair)),
            }),
        ),
        (
            "cdr".to_string(),
            BuiltIn::OneArg(|pair| match pair {
                Expr::Pair(_, box r, ..) => Ok(r.clone()),
                _ => Err(format!("cdr expected pair, found: {}", pair)),
            }),
        ),
        (
            "str-append".to_string(),
            BuiltIn::TwoArg(|l, r| match (l, r) {
                (Expr::String(l, _), Expr::String(r, _)) => Ok(Expr::String(l.clone() + r, None)),
                _ => Err(format!("Expected strings, found: {} and {}", l, r)),
            }),
        ),
        (
            "to-string".to_string(),
            BuiltIn::OneArg(|expr| Ok(Expr::String(format!("{expr}"), None))),
        ),
    ]);
    globals
});

pub fn collect_kws_from_expr(expr: &Expr) -> Result<Vec<String>, CompileError> {
    match expr {
        Expr::Pair(box Expr::Keyword(kw, ..), box rest, ..) => {
            collect_kws_from_expr(rest).map(|mut x| {
                x.insert(0, kw.clone());
                x
            })
        }
        Expr::Nil => Ok(vec![]),
        _ => Err(CompileError {
            message: format!("Invalid keyword list: {}", expr),
            srcloc: extract_srcloc(expr),
        }),
    }
}

pub fn collect_exprs_from_body(expr: &Expr) -> Result<Vec<Expr>, CompileError> {
    match expr {
        Expr::Nil => Ok(vec![]),
        Expr::Pair(box expr, box Expr::Nil, ..) => Ok(vec![expr.to_owned()]),
        Expr::Pair(box expr, next @ box Expr::Pair(..), ..) => {
            collect_exprs_from_body(next).map(|mut x| {
                x.insert(0, expr.to_owned());
                x
            })
        }
        otherwise => Err(CompileError {
            srcloc: extract_srcloc(expr),
            message: format!("tried to collect exprs from body on: {}", otherwise),
        }),
    }
}
fn make_lambda(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let (pairs, unextracted_body) = match expr {
        Expr::Pair(pairs @ box Expr::Nil, body @ box Expr::Pair(..), ..) => (pairs, body),
        Expr::Pair(pairs @ box Expr::Pair(..), body @ box Expr::Pair(..), ..) => (pairs, body),
        otherwise => return comp_err!(expr, "Invalid lambda expression: {}", otherwise),
    };

    let body = collect_exprs_from_body(unextracted_body)?;

    // all kws (including rest)

    let all_kws = collect_kws_from_expr(pairs.borrow())?;

    let dot_kw = all_kws
        .iter()
        .enumerate()
        .find(|(_, kw)| *kw == ".")
        .map(|(index, _)| index);

    if let Some(dot_index) = dot_kw {
        // only valid if it's the second to last argument
        if dot_index + 2 != all_kws.len() {
            return comp_err!(
                expr,
                "rest-dot can only occur as second-to-last argument, but found: ({})",
                all_kws.join(" ")
            );
        }
    };

    let rest_arg = dot_kw.and_then(|index| all_kws.get(index + 1));
    let (kws, _) = all_kws.split_at(dot_kw.unwrap_or(all_kws.len()));

    let mut new_body_chunk = Chunk { code: vec![] };

    // find closing over variables
    compile_many_exprs(body.clone(), &mut new_body_chunk)?;

    chunk
        .code
        .push(VMInstruction::Constant(Expr::LambdaDefinition(
            new_body_chunk,
            rest_arg.cloned(),
            kws.to_vec(),
        )));
    chunk.code.push(VMInstruction::MakeLambda);
    Ok(())
}

fn make_define(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let kw = match expr {
        Expr::Pair(
            box Expr::Pair(box Expr::Keyword(fn_name, ..), fn_args, ..),
            body @ box Expr::Pair(..),
            src_loc,
        ) => {
            // this is a lambda definition
            // kws should contain a fn name and then its args
            make_lambda(
                &Expr::Pair(fn_args.clone(), body.clone(), src_loc.clone()),
                chunk,
            )?;
            Ok(fn_name.clone())
        }
        Expr::Pair(
            box Expr::Keyword(kw, ..),
            box Expr::Pair(box definee, box Expr::Nil, ..),
            ..,
        ) => {
            compile_internal(definee, chunk)?;
            Ok(kw.clone())
        }
        otherwise => {
            return comp_err!(
                expr,
                "definition, expected kw and expr but found: {}",
                otherwise
            )
        }
    }?;
    chunk.code.push(VMInstruction::Define(kw.clone()));
    chunk.code.push(VMInstruction::Constant(Expr::Nil));
    Ok(())
}

fn make_if(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let (pred, consequent, alternate) = match expr {
        Expr::Pair(
            box pred,
            box Expr::Pair(box consequent, box Expr::Pair(box alternate, box Expr::Nil, ..), ..),
            ..,
        ) => (pred, consequent, alternate),
        otherwise => {
            return comp_err!(
                expr,
                "if, expected pred, cons, alt but found: {}",
                otherwise
            )
        }
    };

    let mut pred_chunk = Chunk { code: vec![] };
    compile_internal(pred, &mut pred_chunk)?;

    let mut alt_chunk = Chunk { code: vec![] };
    compile_internal(alternate, &mut alt_chunk)?;

    let mut cons_chunk = Chunk { code: vec![] };
    compile_internal(consequent, &mut cons_chunk)?;

    let end_ip = cons_chunk.code.len();

    let cons_ip = 1 + 1 + alt_chunk.code.len();

    chunk.code.extend_from_slice(&pred_chunk.code);

    chunk.code.push(VMInstruction::CondJumpPop(cons_ip));

    chunk.code.extend_from_slice(&alt_chunk.code);

    chunk.code.push(VMInstruction::Constant(Expr::bool(true)));
    chunk.code.push(VMInstruction::CondJumpPop(end_ip));
    chunk.code.extend_from_slice(&cons_chunk.code);
    Ok(())
}

fn make_and(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let (l, r) = match expr {
        Expr::Pair(box l, box Expr::Pair(box r, box Expr::Nil, ..), ..) => (l, r),
        otherwise => return comp_err!(expr, "and, expected two args but found: {}", otherwise),
    };
    // l + popjmp(r) + jmp(return) + r + return
    let mut r_chunk = Chunk { code: vec![] };
    compile_internal(r, &mut r_chunk)?;
    compile_internal(l, chunk)?;
    chunk.code.push(VMInstruction::CondJump(2));
    chunk.code.push(VMInstruction::Constant(Expr::bool(true)));
    chunk
        .code
        .push(VMInstruction::CondJumpPop(1 + r_chunk.code.len()));
    chunk.code.push(VMInstruction::PopStack);
    chunk.code.extend_from_slice(&r_chunk.code);
    Ok(())
}

fn make_or(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let (l, r) = match expr {
        Expr::Pair(box l, box Expr::Pair(box r, box Expr::Nil, ..), ..) => (l, r),
        otherwise => return comp_err!(expr, "or, expected two args but found: {}", otherwise),
    };
    let mut r_chunk = Chunk { code: vec![] };
    compile_internal(r, &mut r_chunk)?;
    compile_internal(l, chunk)?;
    chunk
        .code
        .push(VMInstruction::CondJump(1 + r_chunk.code.len()));
    chunk.code.push(VMInstruction::PopStack);
    chunk.code.extend_from_slice(&r_chunk.code);
    Ok(())
}

fn make_quote(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let exprs = collect_exprs_from_body(expr)?;
    if let (Some(arg), 1) = (exprs.first(), exprs.len()) {
        chunk.code.push(VMInstruction::Constant(arg.clone()))
    } else {
        return comp_err!(expr, "quote expects 1 arg, but found: {:#?}", exprs);
    }
    Ok(())
}
fn make_display(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    match expr {
        Expr::Pair(box displayee, box Expr::Nil, ..) => {
            compile_internal(displayee, chunk)?;
            chunk.code.push(VMInstruction::Display);
            return Ok(());
        }
        otherwise => {
            return comp_err!(
                expr,
                "Expected one argument for display, but found {}",
                otherwise
            )
        }
    }
}
fn make_apply(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    let exprs = collect_exprs_from_body(expr)?;
    if let (Some(function), Some(args), 2) = (exprs.get(0), exprs.get(1), exprs.len()) {
        compile_internal(function, chunk)?;
        compile_internal(args, chunk)?;
        chunk.code.push(VMInstruction::Apply);
        chunk.code.push(VMInstruction::Call(0));
    } else {
        return comp_err!(expr, "apply expects 2 args, but found: {}", exprs.len());
    }
    Ok(())
}

pub type MacroFn = Arc<dyn Fn(Option<SrcLoc>, &Vec<Expr>) -> Result<Expr, CompileError>>;

pub static SPECIAL_FORMS: Lazy<HashMap<String, fn(&Expr, &mut Chunk) -> CompileResult>> =
    Lazy::new(|| {
        let mut hm = HashMap::<String, fn(&Expr, &mut Chunk) -> CompileResult>::new();
        hm.insert("lambda".to_string(), make_lambda);
        hm.insert("define".to_string(), make_define);
        hm.insert("if".to_string(), make_if);
        hm.insert("and".to_string(), make_and);
        hm.insert("or".to_string(), make_or);
        hm.insert("quote".to_string(), make_quote);
        hm.insert("apply".to_string(), make_apply);
        hm.insert("display".to_string(), make_display);
        hm
    });

pub fn compile_internal(expr: &Expr, chunk: &mut Chunk) -> CompileResult {
    match &expr {
        expr @ Expr::LambdaDefinition(..) | expr @ Expr::Lambda(..) => {
            panic!("Cannot compile a {}", expr)
        }
        Expr::Pair(box Expr::Keyword(kw, ..), box r, ..)
            if let Some(special_form) = SPECIAL_FORMS.get(kw) =>
        {
            special_form(r, chunk)?;
        }
        Expr::Pair(box l, box r, ..) => {
            let exprs = collect_exprs_from_body(r)?;
            if let Expr::Keyword(kw, ..) = l {
                let global_arity = match BUILTIN_FNS.get(kw) {
                    Some(BuiltIn::OneArg(..)) => Some(1),
                    Some(BuiltIn::TwoArg(..)) => Some(2),
                    _ => None,
                };
                if global_arity.is_some_and(|arity| arity != exprs.len()) {
                    return Err(CompileError {
                        srcloc: extract_srcloc(expr),
                        message: format!(
                            "Expected {} arguments for {}, but found {}",
                            global_arity.unwrap(),
                            kw,
                            exprs.len(),
                        ),
                    });
                }
            }
            compile_internal(l, chunk)?;
            for expr in exprs.iter() {
                compile_internal(expr, chunk)?;
            }
            chunk.code.push(VMInstruction::Call(exprs.len()));
        }
        Expr::Keyword(kw, ..) => {
            chunk.code.push(VMInstruction::Lookup(kw.clone()));
        }
        expr @ (Expr::String(..)
        | Expr::Num(..)
        | Expr::Boolean(..)
        | Expr::Quote(..)
        | Expr::Nil) => {
            chunk.code.push(VMInstruction::Constant((*expr).clone()));
        }
    };
    Ok(())
}

pub fn compile_many_exprs(exprs: Vec<Expr>, chunk: &mut Chunk) -> CompileResult {
    return exprs.iter().enumerate().try_fold((), |_, (i, expr)| {
        match compile_internal(expr, chunk) {
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
