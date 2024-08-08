#[cfg(test)]
use crate::parse::ParseInput;

#[test]
fn test_sicp() {
    use crate::vm::run;
    use crate::vm::VM;
    use crate::vm::{get_prelude, prepare_vm, CompilerEnv};
    use include_dir::include_dir;
    use std::collections::HashMap;
    let dir = include_dir!("./sicp");
    let files = dir
        .files()
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
                    &ParseInput {
                        source: src,
                        file_name: Some("test_sicp"),
                    },
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
            .map(|x| (x.path().to_str().unwrap(), x.contents_utf8().unwrap()))
            .collect::<Vec<(&str, &str)>>()
            .len(),
        34
    );

    let expected_logs = HashMap::from([(
        "2.23.scm",
        Ok::<Vec<String>, String>(vec!["57".to_string(), "321".to_string(), "88".to_string()]),
    )]);

    for (file, mut vm) in successful_files {
        let result = run(&mut vm).map(|_| vm);
        let result_log = &result.map(|x| x.log);
        assert!(
            result_log == expected_logs.get(file).unwrap_or(&Ok(vec![])),
            "File: {file} failed when running sicp sols, found: {result_log:?}."
        )
    }
}
