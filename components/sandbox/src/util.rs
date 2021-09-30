use crate::{AnyError, JsValue, Sandbox};

/// Evaluates a standalone Javascript expression, and returns the result as a JSON value.
///
/// If there is an error, Err will be returned.
/// This function is primarily useful for small standalone experiments. Usually, you would want to use the [`Script`](struct.Script.html) struct
/// for more sophisticated Rust->JS interaction.
pub fn eval_json(js_expr: &str) -> Result<JsValue, AnyError> {
    let code = format!(
        "
		function __rust_expr() {{
			return ({expr});
		}}
	",
        expr = js_expr
    );

    let mut sandbox = Sandbox::new(vec![]);
    sandbox.execute("sandboxed.js", code.as_str())?;
    sandbox.call_json("__rust_expr", &JsValue::Null)
}
