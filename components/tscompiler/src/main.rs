use tscompiler::compiler;

pub const JSMOD_CORE_UTIL: &str = include_str!("jack.ts");

fn main() {
    // let compiled = crate::
    let output = compiler::compile_typescript(JSMOD_CORE_UTIL).unwrap();
    println!("{}", output);
}
 