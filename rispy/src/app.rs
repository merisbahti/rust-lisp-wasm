use crate::compile::get_globals;
use crate::expr::Expr;
use crate::vm;
use crate::vm::prepare_vm;
use crate::vm::run;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StackProps {
    stack: Vec<Expr>,
}

#[function_component(Stack)]
pub fn stack(props: &StackProps) -> Html {
    html! {
        <div class="stack">
            { props.stack.clone().into_iter().map(|stack_item|
                html! {<div class="stack-item">{format!("{:?}", stack_item)}</div>}
            ).collect::<Html>() }
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let source_handle = use_state(|| "(+ 1 5)".to_string());
    let source = (*source_handle).clone();

    let vm_handle = use_state(|| vm::prepare_vm(source.clone()));
    let vm = (*vm_handle).clone();

    use_effect_with_deps(
        {
            let vm_handle = vm_handle.clone();
            move |arg: &String| vm_handle.set(vm::prepare_vm(arg.clone()))
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
            let res = prepare_vm(source.clone()).and_then(|mut vm| {
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
                </div>
                <div style="display: flex; flex-direction: column; gap: 8px;">
                    if let Ok(vm) = vm.clone() {
                        <Stack stack={vm.stack} />
                    }
                    if let Err(error) = vm.clone() {
                        <div class="error">{ format!("Error: {:}", error) }</div>
                    }
                </div>
            </div>
        </main>
    }
}
