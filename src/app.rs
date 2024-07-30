use crate::compile::get_globals;
use crate::expr::Expr;
use crate::vm;
use crate::vm::get_prelude;
use crate::vm::prepare_vm;
use crate::vm::run;
use crate::vm::Callframe;
use crate::vm::VM;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StackProps {
    stack: Vec<Expr>,
}

#[function_component]
pub fn Stack(props: &StackProps) -> Html {
    html! {
        <div class="stack">
            if props.stack.is_empty() {
                { "empty stack" }
            }
            { props.stack.clone().into_iter().rev().map(|stack_item|
                html! {<div class="stack-item">{format!("{:?}", stack_item)}</div>}
            ).collect::<Html>() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct CallFramesProps {
    callframe: Callframe,
}

#[function_component]
pub fn Callframes(props: &CallFramesProps) -> Html {
    html! {
        <div class="callframe">
            { props.callframe.chunk.code.clone().into_iter().enumerate().map(|(index,instruction)|
                {
                    let class = if index == props.callframe.ip { "instruction active" } else { "instruction"};
                     html! {<div class={class}>{format!("{:?}", instruction)}</div>}
                }

            ).collect::<Html>() }
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let fib = [
        "(define fib (lambda (n) 
  (fib-iter 1 0 n)))
(define fib-iter (lambda (a b count)
(if (= count 0)
  b
  (fib-iter (+ a b) a (+ count -1)))))
(fib 90)"
            .to_string(),
        "(defmacro (m a) (cons '+ (cons a (cons 2 '()))))
        (m 2)"
            .to_string(),
        r#"(define a "stuff")
            (dprint 1 a "hello")"#
            .to_string(),
        "(if 1 2 3 )".to_string(),
        "(or false true)".to_string(),
        "(and true true)".to_string(),
    ];

    let source_handle = use_state(|| fib.last().cloned().unwrap_or("not found".to_string()));

    let source = (*source_handle).clone();
    fn prepare_with_prelude(src: String) -> Result<VM, String> {
        return prepare_vm(src, None).map(|x| x.0);
        match get_prelude() {
            Ok(prelude) => prepare_vm(src, Some(prelude)).map(|x| x.0),
            Err(err) => Err(format!("Error when compiling prelude: {err}")),
        }
    }

    let vm_handle = use_state(|| prepare_with_prelude(source.clone()));
    let vm = (*vm_handle).clone();

    use_effect_with_deps(
        {
            let vm_handle = vm_handle.clone();
            move |arg: &String| {
                let prepared_vm = prepare_with_prelude(arg.clone());

                vm_handle.set(prepared_vm)
            }
        },
        source.clone(),
    );

    let oninput = Callback::from({
        let name_handle = source_handle.clone();
        move |input_event: InputEvent| {
            let target: HtmlTextAreaElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            web_sys::console::log_1(&target.value().into()); // <- can console the value.
            name_handle.set(target.value());
        }
    });

    let step = Callback::from({
        let vm_result = vm.clone();
        let vm_handle = vm_handle.clone();
        move |_stuff: MouseEvent| {
            let result = vm_result
                .clone()
                .and_then(|mut vm| vm::step(&mut vm, &get_globals()).map(|_| vm));
            vm_handle.set(result.clone());
        }
    });

    let run = Callback::from({
        let source = source.clone();
        let vm_handle = vm_handle.clone();
        move |_stuff: MouseEvent| {
            let prepared_vm = prepare_with_prelude(source.clone());

            let res = prepared_vm.and_then(|mut vm| {
                run(&mut vm, &get_globals())?;
                Ok(vm)
            });
            vm_handle.set(res)
        }
    });

    html! {
        <main>
            <div style="display: flex; gap: 8px;">
                <div style="display: flex; flex-direction: column; gap: 8px">
                    <textarea rows=10 {oninput} value={source} />
                    <div style="display: flex; gap: 8px">
                        <button onclick={step} style="flex-grow: 1;">{ "step" }</button>
                        <button onclick={run} style="flex-grow: 1;">{ "run" }</button>
                    </div>
                    <div class="console">
                        if let Ok(vm) = vm.clone() {
                            { vm.log.into_iter().map(|log| html!{<div>{log}</div>}).collect::<Html>() }
                        }
                    </div>
                </div>
                <div style="display: flex; flex-direction: row; gap: 8px;">
                    if let Ok(vm) = vm.clone() {
                        <Stack stack={vm.stack} />
                    }
                    if let Ok(vm) = vm.clone() {
                        <div class="callframes">
                            { vm.callframes.iter().rev().map(|callframe| html!{<Callframes callframe={callframe.clone()} />}).collect::<Html>() }
                        </div>
                    }
                    if let Err(error) = vm.clone() {
                        <div class="error">{ format!("{:}", error) }</div>
                    }
                </div>
            </div>
        </main>
    }
}
