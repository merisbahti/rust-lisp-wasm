use crate::compile::get_globals;
use crate::vm;
use crate::vm::jit_run;
use crate::vm::prepare_vm;
use crate::vm::run;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[function_component(Stack)]
pub fn stack() -> Html {
    html! { { "stack" } }
}

#[function_component(App)]
pub fn app() -> Html {
    let source_handle = use_state(|| "(+ 1 5)".to_string());
    let source = (*source_handle).clone();

    let vm_handle = use_state(|| vm::prepare_vm(&source_handle.to_string()));
    let vm = (*vm_handle).clone();

    use_effect_with_deps(
        {
            let vm_handle = vm_handle.clone();
            move |arg| vm_handle.set(vm::prepare_vm(&arg))
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
        let new_vm = vm.clone();
        let vm_handle = vm_handle.clone();
        move |_stuff: MouseEvent| {
            match &new_vm.clone() {
                Ok(stuff) => {
                    let mut cloned = stuff.clone();
                    vm::step(&mut cloned, &get_globals());
                    vm_handle.set(Ok(cloned));
                }
                Err(_) => todo!(),
            }
            // Ok(mut stuff) => {
            //
            // vm::step(&mut stuff, &get_globals());
            //     vm_handle.set(Ok(stuff));
            // }

            // err => vm_handle.set(Err("some damn error".to_string())),
        }
    });

    let run = Callback::from({
        let source = source.clone();
        let vm_handle = vm_handle.clone();
        move |_stuff: MouseEvent| {
            let mut vm = match prepare_vm(&source) {
                Ok(res) => res,
                Err(message) => {
                    &vm_handle.set(Err(message.to_string()));
                    return;
                }
            };
            match run(&mut vm, &get_globals()) {
                Ok(_) => vm_handle.set(Ok(vm.clone())),
                Err(message) => vm_handle.set(Err(message.to_string())),
            }
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
                    <Stack />
                    <div>{ format!("{:?}", vm) }</div>
                </div>
            </div>
        </main>
    }
}
