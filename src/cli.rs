use std::path::PathBuf;

use clap::Parser;
use const_format::formatcp;

const VERSION: &str = {
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
    const GIT_COMMIT_DATE: &str = env!("VERGEN_GIT_COMMIT_DATE");

    #[allow(clippy::const_is_empty)]
    if GIT_DESCRIBE.is_empty() {
        formatcp!("{PKG_VERSION}")
    } else {
        formatcp!("{PKG_VERSION} ({GIT_DESCRIBE} {GIT_COMMIT_DATE})")
    }
};

const LONG_VERSION: &str = {
    const RUSTC_SEMVER: &str = env!("VERGEN_RUSTC_SEMVER");
    const RUSTC_HOST_TRIPLE: &str = env!("VERGEN_RUSTC_HOST_TRIPLE");
    const CARGO_FEATURES: &str = env!("VERGEN_CARGO_FEATURES");
    const CARGO_TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");

    formatcp!(
        "{VERSION}

rustc version:       {RUSTC_SEMVER}
rustc host triple:   {RUSTC_HOST_TRIPLE}
cargo features:      {CARGO_FEATURES}
cargo target triple: {CARGO_TARGET_TRIPLE}"
    )
};

#[derive(Debug, Parser)]
#[command(about, version = VERSION, long_version = LONG_VERSION)]
#[command(propagate_version = true)]
pub struct Args {
    #[arg(long, env, value_name = "KEY")]
    pub coinmarketcap_api_key: String,

    #[arg(long, value_name = "PATH", display_order(99))]
    pub data_dir: Option<PathBuf>,
}
