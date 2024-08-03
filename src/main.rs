mod cli;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::Args;
use nix::unistd::Uid;
use reqwest::header::ACCEPT;
use reqwest::Client;
use rss::{ChannelBuilder, ItemBuilder};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{DirBuilder, File},
    io::AsyncWriteExt,
};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let data_dir = data_dir(args.data_dir).await?;

    let price = get_price(&args.coinmarketcap_api_key).await?;
    let xml = generate_rss(price);
    save_rss(&xml, &data_dir).await?;

    Ok(())
}

async fn get_price(key: &str) -> Result<f64> {
    Client::new()
        .get("https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest")
        .query(&[("convert", "USD"), ("symbol", "BTC")])
        .header(ACCEPT, "application/json")
        .header("X-CMC_PRO_API_KEY", key)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?
        .pointer("/data/BTC/quote/USD/price")
        .ok_or(anyhow!("Invalid JSON pointer"))?
        .as_number()
        .ok_or(anyhow!("Not a number"))?
        .as_f64()
        .ok_or(anyhow!("Not a f64"))
}

fn generate_rss(price: f64) -> String {
    let item = ItemBuilder::default()
        .title(Some(format!("${price:.2}")))
        .build();

    ChannelBuilder::default()
        .title("Bitcoin Price")
        .link("https://coinmarketcap.com/currencies/bitcoin")
        .item(item)
        .build()
        .to_string()
}

async fn data_dir(arg: Option<PathBuf>) -> Result<PathBuf> {
    let path = if Uid::effective().is_root() {
        Ok(PathBuf::from("/var/lib/coin-price"))
    } else {
        let default_data_dir = dirs::data_dir().map(|d| d.join(PKG_NAME));
        arg.map(|p| match p.is_dir() {
            true => Ok(p),
            false => Err(anyhow!("given path is not a directory")),
        })
        .transpose()?
        .or(default_data_dir)
        .ok_or(anyhow!(
            "unable to find configuration file. Use the -c flag."
        ))
    }?;

    DirBuilder::new()
        .recursive(true)
        .create(path.join("rss"))
        .await?;
    Ok(path)
}

async fn save_rss(xml: &str, data_dir: &Path) -> Result<()> {
    File::create(data_dir.join("rss").join("bitcoin.xml"))
        .await?
        .write_all(xml.as_bytes())
        .await?;
    Ok(())
}
