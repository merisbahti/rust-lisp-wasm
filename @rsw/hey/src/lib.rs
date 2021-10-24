#![cfg(feature = "alloc")]
mod eval;
use wasm_bindgen::prelude::*;

// Export a `greet` function from Rust to JavaScript, that formats a
// hello message.
#[wasm_bindgen]
pub fn greet(expression: String) -> String {
    return "hello".to_string();
    // let result = eval::eval_from_str(&expression.to_string());
    // return result.to_string();
    // return match result {
    //     Ok(a) => "ok result".to_string(),
    //     Err(e) => "someerror".to_string(),
    // };
}
