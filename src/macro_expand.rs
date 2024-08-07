use std::{collections::HashMap, sync::Arc};

use crate::comp_err;
use crate::compile::{
    collect_exprs_from_body, collect_kws_from_expr, compile_many_exprs, extract_srcloc,
    CompileError,
};
use crate::parse::make_pair_from_vec;
use crate::vm::{run, Callframe};
use crate::{
    compile::MacroFn,
    expr::Expr,
    vm::{get_initial_vm_and_chunk, Chunk, Env, VMInstruction},
};

pub fn make_macro(params: &[String], macro_definition: &Expr) -> MacroFn {
    let params: Vec<String> = params.into();
    Arc::new({
        let macro_definition = macro_definition.clone();
        let all_kws = params.clone();

        move |srcloc, args| {
            let dot_kw = all_kws
                .iter()
                .enumerate()
                .find(|(_, kw)| *kw == ".")
                .map(|(index, _)| index);

            if let Some(dot_index) = dot_kw {
                // only valid if it's the second to last argument
                if dot_index + 2 != all_kws.len() {
                    return comp_err!(
                        &macro_definition,
                        "rest-dot can only occur as second-to-last argument, but found: ({})",
                        all_kws.join(" ")
                    )
                    .map(|_| Expr::Nil);
                }
            };

            let variadic = dot_kw.and_then(|index| all_kws.get(index + 1));
            let (vars, _) = all_kws.split_at(dot_kw.unwrap_or(all_kws.len()));

            let is_variadic = variadic.is_some();

            if is_variadic && args.len() < vars.len() {
                return Err(CompileError {
                    srcloc,
                    message: format!(
                        "wrong number of args, expected at least {} ({}), got: ({})",
                        &macro_definition,
                        vars.len(),
                        vars.join(" "),
                    ),
                });
            }

            if !is_variadic && args.len() != vars.len() {
                return Err(CompileError {
                    srcloc,
                    message: format!(
                        "macro wrong number of args, expected {} ({}), got: ({})",
                        vars.len(),
                        vars.join(" "),
                        args.into_iter()
                            .map(|x| format!("{x}"))
                            .collect::<Vec<String>>()
                            .join(" ")
                    ),
                });
            }

            let mut map = vars
                .iter()
                .cloned()
                .zip(args.clone())
                .collect::<HashMap<String, Expr>>();

            variadic.inspect(|arg_name| {
                let (_, pairs) = args.split_at(vars.len());
                map.insert(arg_name.clone().clone(), make_pair_from_vec(pairs.to_vec()));
            });

            let initial_env = Env { map, parent: None };

            let mut vm = get_initial_vm_and_chunk(initial_env);

            let mut chunk = Chunk { code: vec![] };

            let macro_exprs = collect_exprs_from_body(&macro_definition)?;
            let mut macro_env = {
                let mut macro_env = vec![];
                macro_env.append(&mut vars.to_vec());
                if let Some(rest_arg) = variadic {
                    macro_env.push(rest_arg.clone());
                }
                macro_env
            };
            compile_many_exprs(macro_exprs, &mut chunk, &mut macro_env)?;
            chunk.code.push(VMInstruction::Return);

            let callframe = Callframe {
                ip: 0,
                chunk,
                env: "initial_env".to_string(),
            };
            // add params and args in vm envs (unevaluated)
            vm.callframes.push(callframe);

            match run(&mut vm) {
                Ok(e) => e,
                Err(err) => {
                    return comp_err!(
                        &macro_definition,
                        "Error when running macro expansion: {err}"
                    )
                    .map(|_| Expr::Nil);
                }
            };

            match vm.stack.first() {
                Some(top) if vm.stack.len() == 1 => Ok(top.clone()),
                _ => comp_err!(
                    &macro_definition,
                    "expected one value on the stack, got {:#?}",
                    vm.stack
                )
                .map(|_| Expr::Nil),
            }
        }
    })
}

pub fn macro_expand_one(
    expr: &Expr,
    macros: &mut HashMap<String, MacroFn>,
) -> Result<Expr, CompileError> {
    let argmacros = macros.clone();
    match expr {
        expr @ Expr::Quote(..) => Ok(expr.clone()),
        expr @ Expr::Pair(box Expr::Keyword(quote, ..), ..) if quote == "quote" => Ok(expr.clone()),
        Expr::Pair(box Expr::Keyword(kw, ..), box r, srcloc)
            if let Some(found_macro) = argmacros.get(kw) =>
        {
            let expanded_body = macro_expand_one(r, macros)?;
            let args = collect_exprs_from_body(&expanded_body).map_err(|_| CompileError {
                srcloc: extract_srcloc(expr),
                message: format!(
                    "Error when collecting kws for macro expansion, found: {}",
                    r
                ),
            })?;
            found_macro(srcloc.clone(), &args)
        }

        pair @ Expr::Pair(
            box Expr::Keyword(macroexpand, ..),
            box Expr::Pair(
                box Expr::Pair(
                    box Expr::Keyword(quote, ..),
                    box Expr::Pair(
                        box Expr::Pair(box Expr::Keyword(kw, ..), box r, _),
                        box Expr::Nil,
                        _,
                    ),
                    _,
                ),
                box Expr::Nil,
                srcloc,
            ),
            _,
        ) if let (Some(found_macro), "macroexpand", "quote") =
            (argmacros.get(kw), macroexpand.as_str(), quote.as_str()) =>
        {
            let expanded_body = macro_expand_one(r, macros)?;
            let args = collect_exprs_from_body(&expanded_body).map_err(|_| CompileError {
                srcloc: extract_srcloc(pair),
                message: format!(
                    "Error when collecting kws for macro expansion, found: {}",
                    r
                ),
            })?;
            found_macro(srcloc.clone(), &args)
                .map(|x| Expr::Quote(Box::new(x.clone()), srcloc.clone()))
        }

        Expr::Pair(
            box Expr::Keyword(macroexpand, ..),
            box Expr::Pair(
                box Expr::Pair(
                    box Expr::Keyword(quote, ..),
                    box Expr::Pair(
                        box Expr::Pair(box Expr::Keyword(kw, ..), ..),
                        box Expr::Nil,
                        ..,
                    ),
                    ..,
                ),
                box Expr::Nil,
                _,
            ),
            _,
        ) if let (None, "macroexpand", "quote") =
            (argmacros.get(kw), macroexpand.as_str(), quote.as_str()) =>
        {
            comp_err!(expr, "macro not found: {kw}").map(|_| Expr::Nil)
        }
        Expr::Pair(box Expr::Keyword(macroexpand, ..), rest, ..)
            if let "macroexpand" = (macroexpand.as_str()) =>
        {
            comp_err!(expr, "can't call macroexpand on {rest}").map(|_| Expr::Nil)
        }
        pair @ Expr::Pair(..) => {
            let exprs = collect_exprs_from_body(pair)?;
            let expanded_exprs = exprs
                .into_iter()
                .map(|expr| macro_expand_one(&expr, macros))
                .collect::<Result<Vec<Expr>, CompileError>>()?;
            Ok(make_pair_from_vec(expanded_exprs))
        }
        otherwise => Ok(otherwise.clone()),
    }
}

pub fn macro_expand(
    exprs: Vec<Expr>,
    macros: &mut HashMap<String, MacroFn>,
) -> Result<Vec<Expr>, CompileError> {
    let mut expanded_exprs = Vec::new();
    for expr in exprs {
        let srcloc = extract_srcloc(&expr.clone());
        match expr {
            Expr::Pair(
                box Expr::Keyword(kw, ..),
                box Expr::Pair(
                    box Expr::Pair(box Expr::Keyword(macro_name, ..), box args, ..),
                    box macro_body,
                    ..,
                ),
                ..,
            ) if kw == "defmacro" => {
                let args = collect_kws_from_expr(&args).map_err(|_| CompileError {
                    srcloc,
                    message: "Error when collecting kws for macro definition".to_string(),
                })?;
                let expanded_macro_body = macro_expand_one(&macro_body, macros)?;
                let new_macro = make_macro(&args, &expanded_macro_body);
                macros.insert(macro_name.clone(), new_macro);
            }
            otherwise => expanded_exprs.push(macro_expand_one(&otherwise, macros)?),
        }
    }
    Ok(expanded_exprs)
}

#[test]
fn expansion_noop_test() {
    use crate::parse::parse;
    fn noop_assertion(input: &str) {
        let macros = &mut HashMap::new();
        assert_eq!(
            parse(&crate::parse::ParseInput {
                source: input,
                file_name: Some("expansion_noop_test")
            })
            .and_then(|parsed| macro_expand(parsed, macros).map_err(|err| format!("{err}")))
            .unwrap(),
            parse(&crate::parse::ParseInput {
                source: input,
                file_name: Some("expansion_noop_test")
            })
            .map(|x| x.into_iter().collect::<Vec<Expr>>().clone())
            .unwrap()
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
            parse(&crate::parse::ParseInput {
                source: input,
                file_name: Some("expansion_test")
            })
            .and_then(|parsed| macro_expand(parsed, macros).map_err(|x| x.to_string())),
            Ok(vec![expected.clone()])
        )
    }
    noop_test(
        "
        (defmacro (compile-time-add a b) (+ a b))
        (compile-time-add 5 2)
        ",
        &Expr::num(7.0),
    );
    noop_test(
        "
        (defmacro (compile-time-add a b) (+ a b))
        (compile-time-add (compile-time-add 1 2) 2)
        ",
        &Expr::num(5.0),
    );
}
