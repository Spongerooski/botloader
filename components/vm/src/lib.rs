use serde::Deserialize;

pub mod error_reporter;
pub mod moduleloader;
pub mod validator;
use stores::config::Script;
pub use validator::validate_script;
pub mod vm;

/// Represents a value passed to or from JavaScript.
///
/// Currently aliased as serde_json's Value type.
pub type JsValue = serde_json::Value;

/// Polymorphic error type able to represent different error domains.
pub type AnyError = deno_core::error::AnyError;

pub fn prepend_script_source_header(source: &str, script: Option<&Script>) -> String {
    let mut result = gen_script_source_header(script);
    result.push_str(source);

    result
}

pub fn gen_script_source_header(script: Option<&Script>) -> String {
    match script {
        None => r#"
        const SCRIPT_ID = "";
        "#
        .to_string(),
        Some(h) => {
            format!(
                r#" 
                const SCRIPT_ID = "{}";
                "#,
                h.id,
            )
        }
    }
}
