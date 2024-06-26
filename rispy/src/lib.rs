extern crate cfg_if;
extern crate nom;
extern crate wasm_bindgen;

mod compile;
mod expr;
mod parse;
mod vm;
use cfg_if::cfg_if;
use vm::{prepare_vm, VM};
use wasm_bindgen::prelude::*;

cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Export a `greet` function from Rust to JavaScript, that formats a
// hello message.
#[wasm_bindgen]
pub fn compile(expression: String) -> JsValue {
    let result: Result<VM, String> = prepare_vm(expression);

    match result {
        Ok(a) => serde_wasm_bindgen::to_value(&a).unwrap(),
        Err(e) => JsValue::from_str(format!("{:?}", e).as_str()),
    }
}
