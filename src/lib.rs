#![feature(box_patterns)]
#![feature(map_try_insert)]
#![feature(iterator_try_reduce)]
#![feature(if_let_guard)]
#![feature(assert_matches)]

pub mod compile;
pub mod expr;
pub mod macro_expand;
pub mod parse;
pub mod vm;

#[cfg(test)]
mod tests;
