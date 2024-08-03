use crate::cli::CliArgs;
use anyhow::{anyhow, Result};
use clap::Parser;
#[cfg(unix)]
use const_format::formatcp;
use nix::{unistd::Uid, NixPath};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{DirBuilder, File},
    io::AsyncReadExt,
};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Merge of cli arguments and config files from highest priority to lowest:
/// 1. CLI arguments
/// 2. User config file
/// 3. System config file (Unix-like OS's only)
/// 4. Default values
#[derive(Debug, Default)]
pub struct Config {
    pub data_dir: PathBuf,
    pub coinmarketcap_api_key: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigFile {
    coinmarketcap_api_key: Option<String>,
}

impl Config {
    pub async fn new() -> Result<Self> {
        let mut cfg = Self::default();

        let cli_args = CliArgs::parse();
        let config_file = ConfigFile::merge_all(cli_args.config.as_deref()).await?;
        cfg.merge_with_cfg_file(config_file);
        cfg.merge_with_cli_args(cli_args)?;

        if cfg.data_dir.is_empty() {
            unreachable!() // it's set in merge_with_cli_args(..)
        }
        if cfg.coinmarketcap_api_key.is_empty() {
            Err(anyhow!("api key is not provided"))?
        }

        DirBuilder::new()
            .recursive(true)
            .create(cfg.data_dir.join("rss"))
            .await?;

        Ok(cfg)
    }

    fn merge_with_cfg_file(&mut self, cfg_file: ConfigFile) {
        let ConfigFile {
            coinmarketcap_api_key,
        } = cfg_file;

        if let Some(t) = coinmarketcap_api_key {
            self.coinmarketcap_api_key = t;
        }
    }

    fn merge_with_cli_args(&mut self, cli_args: CliArgs) -> Result<()> {
        let CliArgs {
            config: _,
            data_dir,
            coinmarketcap_api_key,
        } = cli_args;

        self.data_dir = if Uid::effective().is_root() {
            PathBuf::from(formatcp!("/var/lib/{PKG_NAME}"))
        } else {
            data_dir
                .map(|p| match p.is_dir() || p.is_empty() {
                    true => Ok(p),
                    false => Err(anyhow!("given path is not empty or a directory")),
                })
                .transpose()?
                .or(dirs::data_dir().map(|d| d.join(PKG_NAME)))
                .ok_or(anyhow!(
                    "unable to find the data directory path. pass it with --data-dir"
                ))?
        };

        if let Some(t) = coinmarketcap_api_key {
            self.coinmarketcap_api_key = t;
        }

        Ok(())
    }
}

impl ConfigFile {
    async fn merge_all(config_path: Option<&Path>) -> Result<Self> {
        #[cfg(unix)]
        let system_config = Path::new(formatcp!("/etc/{PKG_NAME}/config.toml"));
        let local_config = dirs::config_dir().map(|d| d.join(PKG_NAME).join("config.toml"));
        let user_config = config_path
            .map(|p| match p.is_file() {
                true => Ok(p),
                false => Err(anyhow!("given path is not a file")),
            })
            .transpose()?
            .or(local_config.as_deref())
            .ok_or(anyhow!(
                "unable to find configuration file. pass it with --config"
            ))?;

        let mut cfg = Self::default();
        #[cfg(unix)]
        cfg.merge(system_config).await?;
        cfg.merge(user_config).await?;
        Ok(cfg)
    }

    async fn merge(&mut self, path: &Path) -> Result<()> {
        if path.is_file() {
            let mut buf = String::new();
            File::open(path).await?.read_to_string(&mut buf).await?;
            let Self {
                coinmarketcap_api_key,
            } = toml::from_str(&buf)?;

            if let Some(t) = coinmarketcap_api_key {
                self.coinmarketcap_api_key = Some(t);
            }

            Ok(())
        } else {
            Ok(())
        }
    }
}
