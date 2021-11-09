use structopt::StructOpt;
use tracing_subscriber::{fmt::format::FmtSpan, util::SubscriberInitExt, EnvFilter};

pub mod config;

use crate::config::RunConfig;

pub fn common_init() -> RunConfig {
    match dotenv::dotenv() {
        Ok(_) => {}
        Err(dotenv::Error::Io(_)) => {} // ignore io errors
        Err(e) => panic!("failed loading dotenv file: {}", e),
    }
    init_tracing();

    RunConfig::from_args()
}

fn init_tracing() {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish()
        .init();
}
