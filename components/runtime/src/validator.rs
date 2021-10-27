use std::{rc::Rc, time::Duration};

use deno_core::{op_sync, Extension, JsRuntime, OpState, RuntimeOptions};
use rusty_v8::IsolateHandle;
use tokio::sync::oneshot;
use tracing::info;
use url::Url;

use vm::{
    moduleloader::{ModuleEntry, ModuleManager},
    prepend_script_source_header, AnyError, JsValue,
};

use crate::commonmodels::script::ScriptMeta;

/// Validates a script, making sure it parses correctly and runs the ScriptMeta function to retrieve essential information about this script
pub async fn validate_script(source: String) -> Result<ScriptMeta, AnyError> {
    info!("validating script");

    let module_map = crate::jsmodules::create_module_map();

    let (iso_tx, iso_rx) = oneshot::channel();
    let (result_tx, result_rx) = oneshot::channel();
    let (term_tx, term_rx) = oneshot::channel();

    let current_tokio = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        current_tokio.block_on(async move {
            let result = validator_thread(source, module_map, iso_tx, term_rx).await;
            result_tx.send(result).unwrap();
        })
    });

    let iso_handle = iso_rx.await.unwrap();

    let kill_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        term_tx.send(true).ok();
        iso_handle.terminate_execution();
    });

    let result = result_rx.await.unwrap();

    kill_handle.abort();
    result
}

async fn validator_thread(
    source: String,
    module_map: Vec<ModuleEntry>,
    iso_handle_back: oneshot::Sender<IsolateHandle>,
    term_rx: oneshot::Receiver<bool>,
) -> Result<ScriptMeta, AnyError> {
    let mut rt = JsRuntime::new(RuntimeOptions {
        extensions: vec![Extension::builder()
            .ops(vec![(
                "op_botloader_script_start",
                op_sync(op_script_start),
            )])
            .build()],
        module_loader: Some(Rc::new(ModuleManager { module_map })),
        ..Default::default()
    });

    // time the runtime out after 10 seconds
    let isolate = rt.v8_isolate();
    let iso_handle = isolate.thread_safe_handle();
    iso_handle_back.send(iso_handle).unwrap();

    let module_id = rt
        .load_module(
            &Url::parse("file://user/validating").unwrap(),
            Some(prepend_script_source_header(&source, None)),
        )
        .await?;
    rt.mod_evaluate(module_id);

    tokio::select! {
        _ = rt.run_event_loop(false) =>{},
        _ = term_rx => {
            return Err(anyhow::anyhow!(
                "runaway script detected, script timed out after 10 seconds"
            ));
        }
    }

    let op_state = rt.op_state();

    let r = {
        if let Ok(op) = op_state.try_borrow() {
            match op.try_borrow::<ScriptMeta>() {
                Some(h) => Ok(h.clone()),
                None => Err(anyhow::anyhow!("never called Jack.registerMeta")),
            }
        } else {
            Err(anyhow::anyhow!("failed borrowing op_state"))
        }
    };
    r
}

pub fn op_script_start(state: &mut OpState, args: JsValue, _: ()) -> Result<(), AnyError> {
    let des: ScriptMeta = serde_json::from_value(args)?;
    info!("Set script meta: {:?}", des);
    state.put(des);
    Ok(())
}

// pub struct EmptyModuleLoader {}

// // TODO: make a formal spec for this behaviour
// impl ModuleLoader for EmptyModuleLoader {
//     fn resolve(
//         &self,
//         _op_state: Rc<std::cell::RefCell<OpState>>,
//         mut specifier: &str,
//         referrer: &str,
//         _is_main: bool,
//     ) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
//         info!("resolving module: {} - {}", specifier, referrer);
//         if let Ok(u) = Url::parse(specifier) {
//             return Ok(u);
//         };

//         if specifier == "jack/index" {
//             specifier = "index";
//         }

//         if specifier.starts_with("./") {
//             specifier = specifier.strip_prefix("./").unwrap();
//         }

//         let resolved = Url::parse(format!("file://{}.js", specifier).as_str()).map_err(|e| {
//             anyhow::anyhow!("failed parsing url: {} ({} - {})", e, specifier, referrer)
//         })?;
//         Ok(resolved)
//     }

//     fn load(
//         &self,
//         _op_state: Rc<std::cell::RefCell<OpState>>,
//         module_specifier: &deno_core::ModuleSpecifier,
//         _maybe_referrer: Option<deno_core::ModuleSpecifier>,
//         _is_dyn_import: bool,
//     ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
//         Box::pin(ready(Ok(ModuleSource {
//             code: String::new(),
//             module_url_found: module_specifier.to_string(),
//             module_url_specified: module_specifier.to_string(),
//         })))
//     }
// }

// pub struct ModuleEntry {
//     pub specifier: Url,
//     pub source: &'static str,
// }
