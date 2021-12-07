use std::{cell::RefCell, rc::Rc, time::Duration};

use anyhow::anyhow;
use deno_core::{op_async, Extension, OpState};
use runtime_models::ops::storage::{
    OpStorageBucketEntry, OpStorageBucketEntryId, OpStorageBucketIncr, OpStorageBucketList,
    OpStorageBucketSetIf, OpStorageBucketSetValue, OpStorageBucketSortedList, OpStorageBucketValue,
};
use tracing::{info, instrument};
use twilight_model::id::GuildId;
use vm::AnyError;

use crate::RuntimeContext;

pub fn extension() -> Extension {
    Extension::builder()
        .ops(vec![
            // botloader stuff
            ("op_botloader_bucket_storage_set", op_async(op_storage_set)),
            (
                "op_botloader_bucket_storage_set_if",
                op_async(op_storage_set_if),
            ),
            ("op_botloader_bucket_storage_get", op_async(op_storage_get)),
            ("op_botloader_bucket_storage_del", op_async(op_storage_del)),
            (
                "op_botloader_bucket_storage_list",
                op_async(op_storage_list),
            ),
            (
                "op_botloader_bucket_storage_incr",
                op_async(op_storage_incr),
            ),
            (
                "op_botloader_bucket_storage_sorted_list",
                op_async(op_storage_sorted_list),
            ),
        ])
        .state(move |state| {
            state.put(StorageState {
                doing_limit_check: false,
                hit_limit: false,
                requests_until_limit_check: 0,
            });
            Ok(())
        })
        .build()
}

struct StorageState {
    requests_until_limit_check: u32,
    doing_limit_check: bool,
    hit_limit: bool,
}

pub async fn op_storage_set(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketSetValue,
    _: (),
) -> Result<OpStorageBucketEntry, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_validate_value_len(&args.value)?;
    check_validate_storage_usage(rt_ctx.guild_id, &rt_ctx, state.clone()).await?;

    let entry = rt_ctx
        .bucket_store
        .set(
            rt_ctx.guild_id,
            args.bucket_name,
            args.key,
            args.value.into(),
            args.ttl.map(|ttl| Duration::from_secs(ttl as u64)),
        )
        .await?;

    Ok(entry.into())
}

pub async fn op_storage_set_if(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketSetIf,
    _: (),
) -> Result<Option<OpStorageBucketEntry>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_validate_value_len(&args.value)?;
    check_validate_storage_usage(rt_ctx.guild_id, &rt_ctx, state.clone()).await?;

    let entry = rt_ctx
        .bucket_store
        .set_if(
            rt_ctx.guild_id,
            args.bucket_name,
            args.key,
            args.value.into(),
            args.ttl.map(|ttl| Duration::from_secs(ttl as u64)),
            args.cond.into(),
        )
        .await?;

    Ok(entry.map(Into::into))
}

pub async fn op_storage_get(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketEntryId,
    _: (),
) -> Result<Option<OpStorageBucketEntry>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let entry = rt_ctx
        .bucket_store
        .get(rt_ctx.guild_id, args.bucket_name, args.key)
        .await?;

    Ok(entry.map(Into::into))
}

pub async fn op_storage_del(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketEntryId,
    _: (),
) -> Result<Option<OpStorageBucketEntry>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let entry = rt_ctx
        .bucket_store
        .del(rt_ctx.guild_id, args.bucket_name, args.key)
        .await?;

    if entry.is_some() {
        let mut state = state.borrow_mut();
        let storage_ctx = state.borrow_mut::<StorageState>();

        // re-check in case were at the limti
        storage_ctx.hit_limit = false;
    }

    Ok(entry.map(Into::into))
}

pub async fn op_storage_list(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketList,
    _: (),
) -> Result<Vec<OpStorageBucketEntry>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let limit = if let Some(limit) = args.limit {
        if limit < 100 {
            limit
        } else {
            100
        }
    } else {
        25
    };

    let entries = rt_ctx
        .bucket_store
        .get_many(
            rt_ctx.guild_id,
            args.bucket_name,
            args.key_pattern.unwrap_or_else(|| "%".to_string()),
            args.after.unwrap_or_else(String::new),
            limit,
        )
        .await?;

    Ok(entries.into_iter().map(Into::into).collect())
}

pub async fn op_storage_incr(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketIncr,
    _: (),
) -> Result<OpStorageBucketEntry, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    check_validate_storage_usage(rt_ctx.guild_id, &rt_ctx, state.clone()).await?;

    let entry = rt_ctx
        .bucket_store
        .incr(rt_ctx.guild_id, args.bucket_name, args.key, args.amount)
        .await?;

    Ok(entry.into())
}

pub async fn op_storage_sorted_list(
    state: Rc<RefCell<OpState>>,
    args: OpStorageBucketSortedList,
    _: (),
) -> Result<Vec<OpStorageBucketEntry>, AnyError> {
    let rt_ctx = {
        let state = state.borrow();
        state.borrow::<RuntimeContext>().clone()
    };

    let limit = if let Some(limit) = args.limit {
        if limit < 100 {
            limit
        } else {
            100
        }
    } else {
        25
    };

    let entries = rt_ctx
        .bucket_store
        .sorted_entries(
            rt_ctx.guild_id,
            args.bucket_name,
            args.order.into(),
            args.offset.unwrap_or_default(),
            limit,
        )
        .await?;

    Ok(entries.into_iter().map(Into::into).collect())
}

fn check_validate_value_len(val: &OpStorageBucketValue) -> Result<(), AnyError> {
    match val {
        OpStorageBucketValue::Json(json) => {
            let serialized = serde_json::to_string(json).unwrap();
            if serialized.len() > 1_000_000 {
                Err(anyhow::anyhow!("value too big, max value size is 1MB"))
            } else {
                Ok(())
            }
        }
        OpStorageBucketValue::Double(_) => Ok(()),
    }
}

#[instrument(skip(ctx, state_rc))]
async fn check_validate_storage_usage(
    guild_id: GuildId,
    ctx: &RuntimeContext,
    state_rc: Rc<RefCell<OpState>>,
) -> Result<(), AnyError> {
    let do_check = {
        // fast path
        let mut state = state_rc.borrow_mut();
        let storage_ctx = state.borrow_mut::<StorageState>();

        if !storage_ctx.doing_limit_check {
            if storage_ctx.hit_limit {
                return Err(anyhow!("hit storage limit, delete some entries"));
            } else if storage_ctx.requests_until_limit_check >= 1 {
                // we have more requests until we need to do a check
                storage_ctx.requests_until_limit_check -= 1;
                return Ok(());
            }

            // need to do check
            storage_ctx.doing_limit_check = true;
            true
        } else {
            false
        }
    };

    if do_check {
        info!("doing a storage check");
        let used_storage = ctx.bucket_store.guild_storage_usage_bytes(guild_id).await;

        let mut state = state_rc.borrow_mut();
        let storage_ctx = state.borrow_mut::<StorageState>();
        storage_ctx.doing_limit_check = false;

        match used_storage {
            Err(e) => Err(e.into()),
            Ok(0...10_000_000) => {
                storage_ctx.requests_until_limit_check = 10;
                Ok(())
            }
            // hit the limit
            _ => {
                storage_ctx.hit_limit = true;
                Err(anyhow!("hit storage limit, delete some entries"))
            }
        }
    } else {
        info!("waiting for result of storage check");
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let mut state = state_rc.borrow_mut();
            let storage_ctx = state.borrow_mut::<StorageState>();

            if !storage_ctx.doing_limit_check {
                // done
                if storage_ctx.hit_limit {
                    return Err(anyhow!("hit storage limit, delete some entries"));
                } else {
                    return Ok(());
                }
            }
        }
    }
}
