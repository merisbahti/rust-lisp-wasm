#![feature(box_patterns)]
#![feature(map_try_insert)]
#![feature(iterator_try_reduce)]
#![feature(if_let_guard)]
#![feature(assert_matches)]
mod app;
mod compile;
mod expr;
mod macro_expand;
mod parse;
mod tests;
mod vm;
use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
