use anyhow::{Context, Result};
use reqwest::Client;
use url::Url;

use crate::config::API_BASE_URL;
use crate::models::ApiResponse;

pub async fn fetch_coin_profit(
    client: &Client,
    original_url: &str,
    coin: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<f64> {
    let access_key = extract_access_key(original_url)?;

    let mut api_url = format!(
        "{API_BASE_URL}?page=1&limit=200&access_key={access_key}&user_id&utc=false&coin={coin}&method=summary"
    );

    if let (Some(start), Some(end)) = (start_date, end_date) {
        api_url = format!(
            "{API_BASE_URL}?page=1&limit=200&start_date={start}&end_date={end}&access_key={access_key}&user_id&utc=false&coin={coin}&method=summary"
        );
    }

    let response = client.get(&api_url).send().await?.error_for_status()?;

    let body = response.text().await?;

    let parsed: ApiResponse = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            save_debug_response(coin, &body)?;

            return Err(e.into());
        }
    };

    let mut total = 0.0;

    for item in parsed.data.data {
        let normalized = item.pps_profit.replace(",", "");

        let value = normalized
            .parse::<f64>()
            .context("Failed parsing pps_profit")?;

        total += value;
    }

    Ok(total)
}

fn extract_access_key(url: &str) -> Result<String> {
    let parsed = Url::parse(url)?;

    for (k, v) in parsed.query_pairs() {
        if k == "access_key" {
            return Ok(v.to_string());
        }
    }

    anyhow::bail!("access_key not found")
}

use chrono::Local;
use std::fs;

fn save_debug_response(coin: &str, body: &str) -> Result<()> {
    fs::create_dir_all("debug")?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");

    let filename = format!("debug/{}_{}.json", coin, timestamp);

    fs::write(filename, body)?;

    Ok(())
}
