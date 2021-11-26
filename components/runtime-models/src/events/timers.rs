use serde::Serialize;
use ts_rs::TS;

use crate::util::NotBigU64;

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "bindings/events/IntervalTimerEvent.ts")]
#[serde(rename_all = "camelCase")]
pub struct IntervalTimerEvent {
    pub script_id: NotBigU64,
    pub name: String,
}
