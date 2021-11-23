use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct NotBigU64(pub u64);

impl ts_rs::TS for NotBigU64 {
    const EXPORT_TO: Option<&'static str> = None;
    fn decl() -> String {
        format!("type {}{} = {};", "NotBigU64", "", "number")
    }
    fn name() -> String {
        "number".to_owned()
    }
    fn inline() -> String {
        "number".to_string()
    }
    fn dependencies() -> Vec<ts_rs::Dependency> {
        vec![]
    }
    fn transparent() -> bool {
        false
    }
}
