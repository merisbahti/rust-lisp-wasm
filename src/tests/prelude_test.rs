#[test]
fn prelude_compiles() {
    let env = crate::vm::get_prelude();
    assert_eq!(env.is_ok(), true);
    assert!(
        env.map(|x| x.map.keys().cloned().collect::<Vec<String>>().len())
            .unwrap()
            > 10,
    );
}

#[test]
fn call_macro_defined_in_prelude() {
    let env = crate::vm::get_prelude();
    assert_eq!(env.is_ok(), true);
    assert!(
        env.map(|x| x.map.keys().cloned().collect::<Vec<String>>().len())
            .unwrap()
            > 10,
    );
}
