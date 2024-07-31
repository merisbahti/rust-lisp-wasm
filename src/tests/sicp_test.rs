#[test]
fn test_sicp() {
    use crate::vm::VM;
    use crate::vm::{get_prelude, prepare_vm, CompilerEnv};
    use include_dir::include_dir;
    let dir = include_dir!("./sicp");
    let files = dir
        .files()
        .into_iter()
        .map(|x| (x.path().to_str().unwrap(), x.contents_utf8().unwrap()))
        .collect::<Vec<(&str, &str)>>();

    assert!(files.len() > 10);

    let prelude = &get_prelude().unwrap();

    let files_compiled = files
        .into_iter()
        .map(|(filename, src)| {
            (
                filename,
                prepare_vm(
                    src.to_string(),
                    Some(CompilerEnv {
                        env: prelude.env.clone(),
                        macros: prelude.macros.clone(),
                    }),
                )
                .map(|x| x.0),
            )
        })
        .collect::<Vec<(&str, Result<VM, String>)>>();

    let mut successful_files: Vec<(&str, VM)> = vec![];
    for (file, vm) in files_compiled {
        if let Err(err) = vm {
            panic!("Compiler error when compiling file: {}\n{}", file, err)
        }
        successful_files.push((file, vm.unwrap()))
    }

    assert_eq!(
        dir.files()
            .into_iter()
            .map(|x| (x.path().to_str().unwrap(), x.contents_utf8().unwrap()))
            .collect::<Vec<(&str, &str)>>()
            .len(),
        34
    )
}
