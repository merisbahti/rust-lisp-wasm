use std::{collections::HashMap, sync::Arc};

use crate::compile::{collect_exprs_from_body, collect_kws_from_expr, compile_many_exprs};
use crate::vm::{run, Callframe};
use crate::{
    compile::{get_globals, make_pairs_from_vec, MacroFn},
    expr::Expr,
    vm::{get_initial_vm_and_chunk, Chunk, Env, VMInstruction},
};

pub fn make_macro(
    params: &Vec<String>,
    macro_definition: &Expr,
    macros: &mut HashMap<String, MacroFn>,
) -> MacroFn {
    Arc::new({
        let macro_definition = macro_definition.clone();
        let all_kws = params.clone();
        let macros = macros.clone();

        move |args| {
            let mut macros = macros.clone();
            let dot_kw = all_kws
                .iter()
                .enumerate()
                .find(|(_, kw)| *kw == ".")
                .map(|(index, _)| index);

            if let Some(dot_index) = dot_kw {
                // only valid if it's the second to last argument
                if dot_index + 2 != all_kws.len() {
                    return Err(format!(
                        "rest-dot can only occur as second-to-last argument, but found: {:?}",
                        all_kws
                    ));
                }
            };

            let variadic = dot_kw.and_then(|index| all_kws.get(index + 1));
            let (vars, _) = all_kws.split_at(dot_kw.unwrap_or(all_kws.len()));

            let is_variadic = variadic.is_some();

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

            variadic.clone().inspect(|arg_name| {
                let (_, pairs) = args.split_at(vars.len());
                map.insert(
                    arg_name.clone().clone(),
                    make_pairs_from_vec(pairs.to_vec()),
                );
            });

            let initial_env = Env { map, parent: None };

            let mut vm = get_initial_vm_and_chunk(initial_env);

            let mut chunk = Chunk { code: vec![] };

            let macro_exprs = collect_exprs_from_body(&macro_definition)?;
            compile_many_exprs(macro_exprs, &mut chunk, &get_globals())?;
            chunk.code.push(VMInstruction::Return);

            let callframe = Callframe {
                ip: 0,
                chunk,
                env: "initial_env".to_string(),
            };
            // add params and args in vm envs (unevaluated)
            vm.callframes.push(callframe);

            match run(&mut vm, &get_globals()) {
                Ok(e) => e,
                Err(err) => {
                    return Result::Err(format!("Error when running macro expansion: {err}"))
                }
            };

            match vm.stack.first() {
                Some(top) if vm.stack.len() == 1 => Ok(top.clone()),
                _ => Result::Err(format!(
                    "expected one value on the stack, got {:#?}",
                    vm.stack
                )),
            }
        }
    })
}

pub fn macro_expand_one(
    expr: &Expr,
    macros: &mut HashMap<String, MacroFn>,
) -> Result<Expr, String> {
    let argmacros = macros.clone();
    match expr {
        Expr::Pair(box Expr::Keyword(kw), box r) if let Some(found_macro) = argmacros.get(kw) => {
            let expanded_body = macro_expand_one(r, macros)?;
            let args = collect_exprs_from_body(&expanded_body).map_err(|_| {
                format!(
                    "Error when collecting kws for macro expansion, found: {}",
                    r
                )
            })?;
            found_macro(&args)
        }
        Expr::Pair(box l, box r) => {
            let l_expanded = macro_expand_one(l, macros)?;
            let r_expanded = macro_expand_one(r, macros)?;
            Ok(Expr::Pair(Box::new(l_expanded), Box::new(r_expanded)))
        }
        otherwise => Ok(otherwise.clone()),
    }
}

pub fn macro_expand(
    exprs: Vec<Expr>,
    macros: &mut HashMap<String, MacroFn>,
) -> Result<Vec<Expr>, String> {
    let mut expanded_exprs = Vec::new();
    for expr in exprs {
        match expr {
            Expr::Pair(
                box Expr::Keyword(kw),
                box Expr::Pair(
                    box Expr::Pair(box Expr::Keyword(macro_name), box args),
                    box macro_body,
                ),
            ) if kw == "defmacro" => {
                let args = collect_kws_from_expr(&args)
                    .map_err(|_| "Error when collecting kws for macro definition")?;
                let expanded_macro_body = macro_expand_one(&macro_body, macros)?;
                let new_macro = make_macro(&args, &expanded_macro_body, macros);
                macros.insert(macro_name.clone(), new_macro);
            }
            otherwise => expanded_exprs.push(macro_expand_one(&otherwise, macros)?),
        }
    }
    return Ok(expanded_exprs);
}

#[test]
fn expansion_noop_test() {
    use crate::parse::parse;
    fn noop_assertion(input: &str) {
        let macros = &mut HashMap::new();
        assert_eq!(
            parse(input).and_then(|parsed| macro_expand(parsed, macros)),
            parse(input)
                .map(|x| x
                    .iter()
                    .map(|x| Some(x.clone()))
                    .flatten()
                    .collect::<Vec<Expr>>()
                    .clone())
                .clone()
        )
    }
    noop_assertion("1");
    noop_assertion("(+ 1 2 (+ 2 3))");
    noop_assertion(
        "
        (+ 1 2 (+ 2 3))
        (+ 1 2 (+ 2 3))
        (+ 1 2 (+ 2 3))
        ",
    );
}

#[test]
fn expansion_test() {
    use crate::parse::parse;
    fn noop_test(input: &str, expected: &Expr) {
        let macros = &mut HashMap::new();
        assert_eq!(
            parse(input).and_then(|parsed| macro_expand(parsed, macros)),
            Ok(vec![expected.clone()])
        )
    }
    noop_test(
        "
        (defmacro (compile-time-add a b) (+ a b))
        (compile-time-add 5 2)
        ",
        &Expr::Num(7.0),
    );
    noop_test(
        "
        (defmacro (compile-time-add a b) (+ a b))
        (compile-time-add (compile-time-add 1 2) 2)
        ",
        &Expr::Num(5.0),
    );
}
