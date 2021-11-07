use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use stores::config::{CreateScript, Script};

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub msg: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.field, self.msg))
    }
}

pub fn validate<T: Validator>(val: &T) -> Result<(), Vec<ValidationError>> {
    let mut ctx = ValidationContext { errs: Vec::new() };
    val.validate(&mut ctx);
    if ctx.errs.is_empty() {
        Ok(())
    } else {
        Err(ctx.errs)
    }
}

pub struct ValidationContext {
    errs: Vec<ValidationError>,
}

impl ValidationContext {
    fn push_error(&mut self, field: &str, msg: String) {
        self.errs.push(ValidationError {
            field: field.to_string(),
            msg,
        });
    }
}

pub trait Validator {
    fn validate(&self, ctx: &mut ValidationContext);
}

impl Validator for CreateScript {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_script_name(ctx, &self.name);
        check_script_source(ctx, &self.original_source);
    }
}

impl Validator for Script {
    fn validate(&self, ctx: &mut ValidationContext) {
        check_script_name(ctx, &self.name);
        check_script_source(ctx, &self.original_source);
    }
}

fn check_script_name(ctx: &mut ValidationContext, name: &str) {
    if name.chars().count() > 32 {
        ctx.push_error("name", "name can be max 32 characters long".to_string());
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r#"^[\w_-]*$"#).unwrap();
    }
    if !RE.is_match(name) {
        ctx.push_error(
            "name",
            "name can only contain 'a-z', '-' and '_'".to_string(),
        );
    }
}

fn check_script_source(ctx: &mut ValidationContext, source: &str) {
    if source.len() > 100_000 {
        ctx.push_error("original_source", "source can be max 100KiB".to_string());
    }
}
