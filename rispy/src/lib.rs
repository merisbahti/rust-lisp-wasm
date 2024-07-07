#![feature(box_patterns)]
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

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn step(expression: JsValue) -> JsValue {
    let deserialized: Result<VM, String> = serde_wasm_bindgen::from_value(expression)
        .map_err(|e| e.to_string())
        .and_then(|x| vm::step(x));

    serde_wasm_bindgen::to_value(&deserialized).unwrap()
}
