#![feature(box_patterns)]
#![feature(iterator_try_reduce)]
#![feature(if_let_guard)]
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
