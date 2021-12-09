use std::rc::Rc;

use deno_core::error::JsError;
use regex::Regex;

use crate::LoadedScriptsStore;
use lazy_static::lazy_static;
use tracing::info;

pub(crate) fn create_error_fn(
    loaded_scripts: LoadedScriptsStore,
) -> Rc<deno_core::JsErrorCreateFn> {
    Rc::new(move |mut err: JsError| {
        // let scripts = loaded_scripts.borrow_mut();

        info!("IN CREATE_ERROR_FN, {:#?}", err);

        parse_transform_err_source(&mut err, &loaded_scripts);

        if let Some(stack) = err.stack {
            err.stack = Some(parse_transform_stack(&loaded_scripts, &stack));
        }

        err.into()
    })
}

fn parse_transform_stack(scripts: &LoadedScriptsStore, stack: &str) -> String {
    let mut output = String::new();

    for line in stack.split('\n') {
        if let Some(new_line) = parse_transform_stack_line(scripts, line) {
            output.push_str(&new_line)
        } else {
            output.push_str(line)
        }
        output.push('\n');
    }

    output
}

fn parse_transform_stack_line(scripts: &LoadedScriptsStore, line: &str) -> Option<String> {
    info!("checking line {}", line);
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"\(?(file:///[\w/\.]+):(\d+):(\d+)\)?"#).unwrap();
    }

    let segments = line
        .split(' ')
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();

    info!("segments: {:?}", segments);

    let (main_part, func) = if segments.len() == 2 {
        (segments[1], None)
    } else if segments.len() == 3 {
        (segments[2], Some(segments[1]))
    } else {
        return None;
    };

    let cap = RE.captures(main_part)?;
    info!("Matched regex!");

    let file = cap.get(1)?.as_str();
    let line: u32 = cap.get(2)?.as_str().parse().ok()?;
    let col: u32 = cap.get(3)?.as_str().parse().ok()?;

    info!("file: {}, line: {}, col; {}", file, line, col);

    let (new_file, src_line, src_col) = scripts.get_original_line_col(file, line, col)?;
    let mut output = String::new();
    output.push_str("    at ");
    if let Some(f) = func {
        output.push_str(f);
        output.push(' ');
    }

    output.push_str(&format!("({}:{}:{})", new_file, src_line, src_col));
    Some(output)
}

fn parse_transform_err_source(err: &mut JsError, scripts: &LoadedScriptsStore) -> Option<()> {
    let res = err.script_resource_name.clone()?;
    let line = err.line_number?;
    let col = err.start_column?;

    let (new_file, src_line, src_col) =
        scripts.get_original_line_col(&res, line as u32, col as u32)?;

    err.script_resource_name = Some(new_file);
    err.line_number = Some(src_line as i64);
    err.start_column = Some(src_col as i64);
    err.end_column = Some(src_col as i64);

    Some(())
}
