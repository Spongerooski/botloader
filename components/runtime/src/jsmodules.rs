use url::Url;
use vm::moduleloader::ModuleEntry;

macro_rules! include_js {
    ($f:tt) => {
        include_str!(concat!(env!("OUT_DIR"), concat!("/js/", $f)));
    };
}

// we could import the whole folder but these need to be ordered so for now we just hardcode them
pub const CORE_MODULES: &[(&str, &str)] = &[
    ("core_util", include_js!("core_util.js")),
    ("commonmodels", include_js!("commonmodels.js")),
    ("bot", include_js!("bot.js")),
    ("op_wrappers", include_js!("op_wrappers.js")),
    ("timers", include_js!("timers.js")),
    ("index", include_js!("index.js")),
];

pub fn create_module_map() -> Vec<ModuleEntry> {
    CORE_MODULES
        .iter()
        .map(|(name, source)| ModuleEntry {
            source: *source,
            specifier: Url::parse(format!("file://{}.js", name).as_str()).unwrap(),
        })
        .collect()
}

// pub async fn load_core_modules(sbox: &mut sandbox::Sandbox) {
//     for (name, source) in CORE_MODULES {
//         sbox.add_eval_module(name.to_string(), source.to_string())
//             .await
//             .unwrap();
//     }
// }
