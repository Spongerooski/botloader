pub mod moduleloader;
use std::{cell::RefCell, rc::Rc};

use deno_core::v8_set_flags;
use stores::config::Script;
use tscompiler::CompiledItem;
pub mod vm;

/// Represents a value passed to or from JavaScript.
///
/// Currently aliased as serde_json's Value type.
pub type JsValue = serde_json::Value;

/// Polymorphic error type able to represent different error domains.
pub type AnyError = deno_core::error::AnyError;

pub static BOTLOADER_CORE_SNAPSHOT: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/BOTLOADER_SNAPSHOT.bin"));

pub fn prepend_script_source_header(source: &str, script: Option<&Script>) -> String {
    let mut result = gen_script_source_header(script);
    result.push_str(source);
    result.push_str("\nscript.run();");

    result
}

const SCRIPT_HEADER_NUM_LINES: u32 = 4;

#[test]
fn hmm() {
    let res = gen_script_source_header(None);
    assert!(res.lines().count() == 4);
}

fn gen_script_source_header(script: Option<&Script>) -> String {
    match script {
        None => r#"
        import {Script} from "script";
        const script = new Script(0);
        "#
        .to_string(),
        Some(h) => {
            format!(
                r#"
                import {{Script}} from "script";
                const script = new Script({});
                "#,
                h.id
            )
        }
    }
}

pub fn init_v8_flags(v8_flags: &[String]) {
    let v8_flags_includes_help = v8_flags
        .iter()
        .any(|flag| flag == "-help" || flag == "--help");

    // Keep in sync with `standalone.rs`.
    let v8_flags = vec!["UNUSED_BUT_NECESSARY_ARG0".to_owned()]
        .into_iter()
        .chain(v8_flags.iter().cloned())
        .collect::<Vec<_>>();
    let unrecognized_v8_flags = v8_set_flags(v8_flags)
        .into_iter()
        .skip(1)
        .collect::<Vec<_>>();

    if !unrecognized_v8_flags.is_empty() {
        for f in unrecognized_v8_flags {
            eprintln!("error: V8 did not recognize flag '{}'", f);
        }
        std::process::exit(1);
    }
    if v8_flags_includes_help {
        std::process::exit(0);
    }
}

#[derive(Clone, Debug)]
pub struct ScriptLoad {
    pub compiled: CompiledItem,
    pub inner: Script,
}

impl ScriptLoad {
    fn get_original_line_col(&self, line_no: u32, col: u32) -> Option<(u32, u32)> {
        self.compiled
            .source_map
            .lookup_token(line_no - SCRIPT_HEADER_NUM_LINES, col)
            .map(|token| (token.get_src_line() + 1, token.get_src_col()))
    }
}

#[derive(Clone)]
pub struct LoadedScriptsStore {
    loaded_scripts: Rc<RefCell<Vec<ScriptLoad>>>,
}

impl LoadedScriptsStore {
    pub fn get_original_line_col(
        &self,
        res: &str,
        line: u32,
        col: u32,
    ) -> Option<(String, u32, u32)> {
        if let Some(guild_script_name) = Self::get_guild_script_name(res) {
            if let Some(script_load) = self
                .loaded_scripts
                .borrow()
                .iter()
                .find(|v| v.inner.name == guild_script_name)
                .cloned()
            {
                if let Some((line, col)) = script_load.get_original_line_col(line, col) {
                    return Some((format!("guild_scripts/{}.ts", guild_script_name), line, col));
                }
            }
        }

        None
    }

    pub fn get_guild_script_name(res: &str) -> Option<&str> {
        if let Some(stripped) = res.strip_prefix("file:///guild_scripts/") {
            if let Some(end_trimmed) = stripped.strip_suffix(".js") {
                return Some(end_trimmed);
            }
        }

        None
    }
}
