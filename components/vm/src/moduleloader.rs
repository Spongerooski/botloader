use deno_core::{ModuleLoader, ModuleSource, OpState};
use futures::future::ready;
use std::rc::Rc;
use tracing::info;
use url::Url;

pub struct ModuleManager {
    // loaded_modules: Vec<url::Url>,
    pub module_map: Vec<ModuleEntry>,
}

// TODO: make a formal spec for this behaviour
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

        if specifier == "bot/index" {
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
        info!("loading module: {}", module_specifier.to_string());

        let f = match self
            .module_map
            .iter()
            .find(|e| e.specifier == *module_specifier)
        {
            Some(e) => ready(Ok(ModuleSource {
                code: e.source.to_string(),
                module_url_found: module_specifier.to_string(),
                module_url_specified: module_specifier.to_string(),
            })),
            None => ready(Err(anyhow::anyhow!(
                "failed finding module {:?}",
                module_specifier
            ))),
        };

        Box::pin(f)
    }
}

pub struct ModuleEntry {
    pub specifier: Url,
    pub source: &'static str,
}
