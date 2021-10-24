extern crate cfg_if;
extern crate nom;
extern crate wasm_bindgen;

mod eval;
use cfg_if::cfg_if;
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
pub fn evaluate_symbolic_string(expression: String) -> String {
    let result = eval::eval_from_str(&expression.to_string());
    return match result {
        Ok(a) => match a {
            eval::Expr::Constant(cons) => match cons {
                eval::Atom::Num(nr) => (nr).to_string(),
                _ => "non-number constant".to_string(),
            },
            _ => "non-constant".to_string(),
        },
        Err(e) => "someerror".to_string(),
    };
}
