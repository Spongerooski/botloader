use tscompiler::compile_typescript;

use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/ts/*");

    // let files = vec!["op_wrappers", "core_util", "jack"];

    let entries = std::fs::read_dir("./src/ts/")
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    let mut loaded_files = Vec::new();
    for file in &entries {
        let filename = file.file_name().unwrap().to_str().unwrap();
        if filename.ends_with(".d.ts") || !filename.ends_with(".ts") {
            continue;
        }

        let mut result = File::open(file).unwrap();
        let mut contents = String::new();
        result.read_to_string(&mut contents).unwrap();

        loaded_files.push((filename.strip_suffix(".ts").unwrap(), contents));
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let _ = fs::create_dir(out_dir.join("js/"));

    for (name, file) in loaded_files {
        let output = compile_typescript(&file).unwrap();
        fs::write(out_dir.join(format!("js/{}.js", name)), output).unwrap();
    }
}
