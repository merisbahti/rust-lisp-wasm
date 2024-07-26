#[test]
fn prelude_compiles() {
    let env = crate::vm::get_prelude();
    assert_eq!(env.is_ok(), true);
    assert_eq!(
        env.map(|x| x.map.keys().cloned().collect::<Vec<String>>()),
        Ok(vec![])
    );
}
