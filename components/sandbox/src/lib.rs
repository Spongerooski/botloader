pub use sandbox::Sandbox;
pub use util::eval_json;

/// Represents a value passed to or from JavaScript.
///
/// Currently aliased as serde_json's Value type.
pub type JsValue = serde_json::Value;

/// Polymorphic error type able to represent different error domains.
///
/// Currently reusing [anyhow::Error](../anyhow/enum.Error.html), this type may change slightly in the future depending on js-sandbox's needs.
// use through deno_core, to make sure same version of anyhow crate is used
pub type AnyError = deno_core::error::AnyError;

mod sandbox;
mod util;
