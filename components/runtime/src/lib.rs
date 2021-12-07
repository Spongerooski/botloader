use std::sync::Arc;

use contrib_manager::LoadedScript;
use deno_core::{op_sync, Extension, OpState};
use guild_logger::{GuildLogger, LogEntry};
use runtime_models::script::ScriptMeta;
use stores::bucketstore::BucketStore;
use tokio::sync::mpsc;
use tracing::info;
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::id::GuildId;
use vm::{
    vm::{VmCommand, VmRole},
    AnyError, JsValue,
};

pub mod contrib_manager;
pub mod dispatchevents;
pub mod extensions;
pub mod jsmodules;
pub mod validator;

pub use validator::validate_script;

pub fn create_extensions(ctx: RuntimeContext) -> Vec<Extension> {
    let core_extension = Extension::builder()
        .ops(vec![
            // botloader stuff
            ("op_botloader_script_start", op_sync(op_script_start)),
            // discord stuff
        ])
        .state(move |state| {
            state.put(ctx.clone());
            Ok(())
        })
        .middleware(Box::new(|name, b| match name {
            // we have our own custom print function
            "op_print" => op_sync(disabled_op),
            _ => b,
        }))
        .build();

    vec![
        core_extension,
        extensions::storage::extension(),
        extensions::discord::extension(),
        extensions::console::extension(),
    ]
}

pub fn in_mem_source_load_fn(src: &'static str) -> Box<dyn Fn() -> Result<String, AnyError>> {
    Box::new(move || Ok(src.to_string()))
}

pub fn dummy_op(_state: &mut OpState, _args: JsValue, _: ()) -> Result<(), AnyError> {
    Err(anyhow::anyhow!(
        "unimplemented, this op is not implemented yet"
    ))
}

pub fn disabled_op(_state: &mut OpState, _args: JsValue, _: ()) -> Result<(), AnyError> {
    Err(anyhow::anyhow!("this op is disabled"))
}

#[derive(Clone)]
pub struct RuntimeContext {
    pub guild_id: GuildId,
    pub bot_state: Arc<InMemoryCache>,
    pub dapi: Arc<twilight_http::Client>,
    pub role: VmRole,
    pub contrib_manager_handle: contrib_manager::ContribManagerHandle,
    pub guild_logger: GuildLogger,
    pub vm_cmd_dispatch_tx: mpsc::UnboundedSender<VmCommand>,
    pub bucket_store: Arc<dyn BucketStore + Send + Sync + 'static>,
}

pub fn op_script_start(state: &mut OpState, args: JsValue, _: ()) -> Result<(), AnyError> {
    let des: ScriptMeta = serde_json::from_value(args)?;

    info!(
        "running script! {}, commands: {}",
        des.script_id.0,
        des.commands.len() + des.command_groups.len()
    );

    let ctx = state.borrow::<RuntimeContext>();

    if let Err(err) = validate_script_meta(&des) {
        // error!(%err, "script meta valication failed");
        ctx.guild_logger.log(LogEntry::script_error(
            ctx.guild_id,
            format!("script meta validation failed: {}", err),
            format!("{}", des.script_id),
            None,
        ));
        return Err(err);
    }

    ctx.contrib_manager_handle.send(LoadedScript {
        guild_id: ctx.guild_id,
        meta: des,
        vm_cmd_dispath_tx: ctx.vm_cmd_dispatch_tx.clone(),
    });

    Ok(())
}

pub(crate) fn validate_script_meta(meta: &ScriptMeta) -> Result<(), anyhow::Error> {
    let mut outbuf = String::new();

    for command in &meta.commands {
        if let Err(verrs) = validation::validate(command) {
            for verr in verrs {
                outbuf.push_str(format!("\ncommand {}: {}", command.name, verr).as_str());
            }
        }
    }

    for group in &meta.command_groups {
        if let Err(verrs) = validation::validate(group) {
            for verr in verrs {
                outbuf.push_str(format!("\ncommand group {}: {}", group.name, verr).as_str());
            }
        }
    }

    if outbuf.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("failed validating script: {}", outbuf))
    }
}
