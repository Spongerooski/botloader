use serde::Deserialize;
use stores::config::ScriptContext;

pub mod error_reporter;
pub mod moduleloader;
pub mod validator;
pub use validator::validate_script;
pub mod vm;

/// Represents a value passed to or from JavaScript.
///
/// Currently aliased as serde_json's Value type.
pub type JsValue = serde_json::Value;

/// Polymorphic error type able to represent different error domains.
pub type AnyError = deno_core::error::AnyError;

#[derive(Deserialize, Debug, Clone)]
pub enum ContextType {
    Guild,
    Channel,
}

impl Default for ContextType {
    fn default() -> Self {
        Self::Guild
    }
}

pub fn prepend_script_source_header(source: &str, script: Option<&ContextScript>) -> String {
    let mut result = gen_script_source_header(script);
    result.push_str(source);

    result
}

pub fn gen_script_source_header(script: Option<&ContextScript>) -> String {
    match script {
        None => r#"
        const SCRIPT_ID = "";
        const SCRIPT_CONTEXT_ID = "";
        "#
        .to_string(),
        Some(h) => {
            format!(
                r#" 
                const SCRIPT_ID = "{}";
                const SCRIPT_CONTEXT_ID = "{}/{}";
                "#,
                h.0.id,
                h.0.id,
                h.1.module_name(),
            )
        }
    }
}

pub type ContextScript = (stores::config::Script, ScriptContext);
pub type ContextScriptId = (u64, ScriptContext);

trait ModuleNamer {
    fn module_name(&self) -> String;
}

impl ModuleNamer for ScriptContext {
    fn module_name(&self) -> String {
        match self {
            Self::Guild => "guild".to_string(),
            Self::Channel(c) => format!("channel/{}", c),
            Self::Role(r) => format!("role/{}", r),
        }
    }
}
