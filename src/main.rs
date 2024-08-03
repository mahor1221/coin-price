mod cli;
mod config;

use anyhow::{anyhow, Result};
use config::Config;
use reqwest::header::ACCEPT;
use reqwest::Client;
use rss::{ChannelBuilder, ItemBuilder};
use serde_json::Value;
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cfg = Config::new().await?;
    let price = get_price(&cfg.coinmarketcap_api_key).await?;
    let xml = generate_rss(price);
    save_rss(&xml, &cfg.data_dir).await
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

async fn save_rss(xml: &str, data_dir: &Path) -> Result<()> {
    File::create(data_dir.join("rss").join("bitcoin.xml"))
        .await?
        .write_all(xml.as_bytes())
        .await?;
    Ok(())
}
