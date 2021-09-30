use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use deno_core::ModuleId;
use deno_core::{Extension, ModuleLoader};
use deno_core::{JsRuntime, OpState, RuntimeOptions, ZeroCopyBuf};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::info;
use url::Url;

use crate::{AnyError, JsValue};

pub struct Sandbox {
    pub runtime: JsRuntime,
    last_rid: u32,
}

const DEFAULT_FILENAME: &str = "sandboxed.js";

impl Sandbox {
    pub fn new(extensions: Vec<Extension>) -> Self {
        let options = RuntimeOptions {
            extensions,
            module_loader: Some(Rc::new(ModuleManager {})),
            ..Default::default()
        };

        let mut runtime = JsRuntime::new(options);
        // runtime.execute(js_filename, &js_code)?;
        runtime.register_op("__rust_return", deno_core::op_sync(Self::op_return));
        runtime.sync_ops_cache();

        Sandbox {
            runtime,
            last_rid: 0,
        }
    }

    fn op_return(
        state: &mut OpState,
        args: JsValue,
        _buf: Option<ZeroCopyBuf>,
    ) -> Result<JsValue, AnyError> {
        let entry = ResultResource { json_value: args };
        let resource_table = &mut state.resource_table;
        let _rid = resource_table.add(entry);
        //assert_eq!(rid, self.last_rid);

        Ok(serde_json::Value::Null)
    }
}

impl Sandbox {
    /// Invokes a JavaScript function.
    ///
    /// Passes a single argument `args` to JS by serializing it to JSON (using serde_json).
    /// Multiple arguments are currently not supported, but can easily be emulated using a `Vec` to work as a JSON array.
    pub fn call<P, R>(&mut self, fn_name: &str, args: &P) -> Result<R, AnyError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let json_args = serde_json::to_value(args)?;
        let json_result = self.call_json(fn_name, &json_args)?;
        let result: R = serde_json::from_value(json_result)?;

        Ok(result)
    }

    pub(crate) fn call_json(&mut self, fn_name: &str, args: &JsValue) -> Result<JsValue, AnyError> {
        // undefined will cause JSON serialization error, so it needs to be treated as null
        let js_code = format!(
            "{{
            let __rust_result = {f}({a});
			if (typeof __rust_result === 'undefined')
				__rust_result = null;
            
			Deno.core.opSync(\"__rust_return\", __rust_result);  
            }}",
            f = fn_name,
            a = args
        );

        self.runtime.execute(DEFAULT_FILENAME, &js_code)?;

        let state_rc = self.runtime.op_state();
        let mut state = state_rc.borrow_mut();
        let table = &mut state.resource_table;

        // Get resource, and free slot (no longer needed)
        let entry: Rc<ResultResource> = table
            .take(self.last_rid)
            .expect("Resource entry must be present");
        let extracted =
            Rc::try_unwrap(entry).expect("Rc must hold single strong ref to resource entry");
        self.last_rid += 1;

        Ok(extracted.json_value)
    }

    pub fn add_state_data<T: 'static>(&mut self, data: T) {
        let state_rc = self.runtime.op_state();
        let mut state = state_rc.borrow_mut();
        state.put(data);
    }

    pub fn execute(&mut self, filename: &str, source: &str) -> Result<(), AnyError> {
        self.runtime.execute(filename, source)
    }

    pub async fn add_eval_module(
        &mut self,
        name: String,
        source: String,
    ) -> Result<ModuleId, AnyError> {
        let id = self
            .runtime
            .load_module(
                &Url::parse(format!("file://{}.js", name).as_str()).unwrap(),
                Some(source),
            )
            .await?;

        // TODO: should we also await the futures?
        // with
        let mut r = self.runtime.mod_evaluate(id);
        self.runtime.run_event_loop(false).await?;

        if let Ok(Some(result)) = r.try_next() {
            result?;
        };

        Ok(id)
    }

    pub async fn add_eval_modules(
        &mut self,
        modules: Vec<(String, String)>,
    ) -> Result<Vec<ModuleId>, AnyError> {
        let mut ids = Vec::new();
        for (name, source) in modules {
            let id = self
                .runtime
                .load_module(
                    &Url::parse(format!("file://{}.js", name).as_str()).unwrap(),
                    Some(source),
                )
                .await?;

            ids.push(id);
        }

        for id in &ids {
            let mut r = self.runtime.mod_evaluate(*id);

            self.runtime.run_event_loop(false).await?;

            if let Ok(Some(result)) = r.try_next() {
                result?;
            };
        }

        Ok(ids)
    }

    /// A utility function that runs provided future concurrently with the event loop.
    ///
    /// Useful when using a local inspector session.
    pub async fn with_event_loop<'a, T>(
        &mut self,
        mut fut: Pin<Box<dyn Future<Output = T> + 'a>>,
    ) -> T {
        loop {
            tokio::select! {
              result = &mut fut => {
                return result;
              }
              _ = self.runtime.run_event_loop(false) => {}
            };
        }
    }
}

#[derive(Debug)]
struct ResultResource {
    json_value: JsValue,
}

// Type that is stored inside Deno's resource table
impl deno_core::Resource for ResultResource {
    fn name(&self) -> Cow<str> {
        "__rust_Result".into()
    }
}

struct ModuleManager {
    // loaded_modules: Vec<url::Url>,
}

impl ModuleLoader for ModuleManager {
    fn resolve(
        &self,
        _op_state: Rc<std::cell::RefCell<OpState>>,
        mut specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
        info!("resolving module: {} - {}", specifier, referrer);
        if let Ok(u) = Url::parse(specifier) {
            return Ok(u);
        };

        if specifier == "jack/index" {
            specifier = "index";
        }

        if specifier.starts_with("./") {
            specifier = specifier.strip_prefix("./").unwrap();
        }

        let resolved = Url::parse(format!("file://{}.js", specifier).as_str()).map_err(|e| {
            anyhow::anyhow!("failed parsing url: {} ({} - {})", e, specifier, referrer)
        })?;
        Ok(resolved)
    }

    fn load(
        &self,
        _op_state: Rc<std::cell::RefCell<OpState>>,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<deno_core::ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        todo!(
            "module loading isn't implemented yet!: {}",
            module_specifier
        )
    }
}
