use std::time::Duration;

use configstore::ScriptContext;
use deno_core::{op_sync, Extension, OpState};
use jack_sandbox::{AnyError, JsValue, Sandbox};
use serde::Deserialize;
use tokio::sync::oneshot;
use tracing::info;

use crate::jsextensions::jsmodules;

mod commonmodels;
pub mod error_reporter;
pub mod runtime;

mod jsextensions;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptHeader {
    pub name: String,
    pub context: ContextType,

    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
}

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

/// Validates a script, making sure it parses correctly and runs the ScriptMeta function to retrieve essential information about this script
pub async fn validate_script(source: &str) -> Result<ScriptHeader, AnyError> {
    info!("validating script");

    let mut sandbox = Sandbox::new(vec![Extension::builder()
        .ops(vec![("op_jack_register_meta", op_sync(op_set_meta))])
        .build()]);

    jsmodules::load_core_modules(&mut sandbox).await;

    // time the runtime out after 10 seconds
    let isolate = sandbox.runtime.v8_isolate();
    let iso_handle = isolate.thread_safe_handle();

    let (term_tx, mut term_rx) = oneshot::channel();

    let kill_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        term_tx.send(true).ok();
        iso_handle.terminate_execution();
    });

    // run the user script
    sandbox
        .add_eval_module(
            "user/validating".to_string(),
            prepend_script_source_header(source, None),
        )
        .await?;

    kill_handle.abort();

    if term_rx.try_recv().is_ok() {
        return Err(anyhow::anyhow!(
            "runaway script detected, script timed out after 10 seconds"
        ));
    }

    let op_state = sandbox.runtime.op_state();

    let r = {
        if let Ok(op) = op_state.try_borrow() {
            match op.try_borrow::<ScriptHeader>() {
                Some(h) => Ok(h.clone()),
                None => Err(anyhow::anyhow!("never called Jack.registerMeta")),
            }
        } else {
            Err(anyhow::anyhow!("failed borrowing op_state"))
        }
    };

    r
}

pub fn op_set_meta(state: &mut OpState, args: JsValue, _: ()) -> Result<(), AnyError> {
    let des: ScriptHeader = serde_json::from_value(args)?;
    info!("Set script header: {:?}", des);
    state.put(des);
    Ok(())
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

pub type ContextScript = (configstore::Script, ScriptContext);
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
