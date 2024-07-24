use crate::vm;

use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let name_handle = use_state(|| "(+ 1 5)".to_string());
    let name = (*name_handle).clone();
    let oninput = Callback::from({
        let name = name_handle.clone();
        move |input_event: InputEvent| {
            let target: HtmlTextAreaElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            web_sys::console::log_1(&target.value().into()); // <- can console the value.
            name.set(target.value());
        }
    });
    html! {
        <main>
            <div style="display: flex;">
                <textarea {oninput} value={name} />
                <p>{"name: "}<h5>{&*name_handle}</h5></p>
                <div style="display: flex; flex-direction: column; gap: 8px;">
                    <div>
                        {format!("{:?}", vm::jit_run(&name_handle.to_string()))}
                    </div>
                    <div>
                        {format!("{:?}", vm::prepare_vm(&name_handle.to_string()))}
                    </div>
                </div>
            </div>
        </main>
    }
}
