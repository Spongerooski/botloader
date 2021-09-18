use std::sync::{Arc, Mutex};

use swc_common::{
    self, chain,
    errors::{Diagnostic, Emitter, Handler},
    sync::Lrc,
    FileName, Globals, Mark, SourceMap,
};
// use swc_ecmascript::ast::Module;
use swc_ecmascript::{
    ast::EsVersion,
    codegen::{text_writer::JsWriter, Emitter as CodeEmitter},
    parser::TsConfig,
    transforms::{
        compat::{self, es2020::export_namespace_from},
        fixer, helpers, resolver_with_mark,
        typescript::strip,
    },
};
use swc_ecmascript::{
    codegen::Config as CodeGenConfig,
    parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax},
    visit::FoldWith,
};

pub fn compile_typescript(input: &str) -> Result<String, Vec<Diagnostic>> {
    match compile_typescript_inner(input) {
        Err(e) => Err(e),
        Ok(str) => Ok(str),
        // None => Ok(String::new()), // TODO: Proper error here? not sure how we can encounter this thoughhh
    }
}

fn compile_typescript_inner(input: &str) -> Result<String, Vec<Diagnostic>> {
    let mut result_buf = Vec::new();

    swc_common::GLOBALS.set(&Globals::new(), || {
        helpers::HELPERS.set(&helpers::Helpers::default(), || {
            let global_mark = Mark::fresh(Mark::root());

            let cm: Lrc<SourceMap> = Default::default();

            // let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
            let errs = Arc::new(Mutex::new(Vec::new()));
            let err_collector = CollectingEmitter {
                messages: errs.clone(),
            };
            let handler = Handler::with_emitter(true, false, Box::new(err_collector));

            let fm = cm.new_source_file(FileName::Custom("script.ts".into()), input.into());

            let lexer = Lexer::new(
                Syntax::Typescript(TsConfig {
                    ..Default::default()
                }),
                EsVersion::Es2020,
                StringInput::from(&*fm),
                None,
            );

            let capturing = Capturing::new(lexer);

            let mut parser = Parser::new_from(capturing);

            for e in parser.take_errors() {
                e.into_diagnostic(&handler).emit();
            }

            let mut module = match parser.parse_module() {
                Ok(m) => m,
                Err(e) => {
                    e.into_diagnostic(&handler).emit();
                    let errs = errs.lock().unwrap();
                    return Err(errs.clone());
                }
            };

            let mut pass = chain!(
                compat::es2021::es2021(),
                strip(),
                export_namespace_from(),
                resolver_with_mark(global_mark),
                compat::reserved_words::reserved_words(),
                fixer(None),
            );

            module = module.fold_with(&mut pass);

            {
                let writer = JsWriter::new(cm.clone(), "\n", &mut result_buf, None);

                let mut emitter = CodeEmitter {
                    cfg: CodeGenConfig {
                        ..Default::default()
                    },
                    cm,
                    comments: None,
                    wr: Box::new(writer),
                };

                // TODO: handle the io error? how would there be a io error since its a in mem buffer though?
                emitter.emit_module(&module).unwrap();
            }

            // i really hope this dosen't produce any invalid utf8 stuff :eyes:
            Ok(String::from_utf8(result_buf).unwrap())
        })
    })
}

struct CollectingEmitter {
    messages: Arc<Mutex<Vec<Diagnostic>>>,
}

impl Emitter for CollectingEmitter {
    fn emit(&mut self, db: &swc_common::errors::DiagnosticBuilder<'_>) {
        let mut messages = self.messages.lock().unwrap();
        // let mut_brw = self.messages.borrow_mut();
        messages.push((**db).clone());
        // println!("[SWC]: {:?}", db);
    }
}
