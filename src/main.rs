use anyhow::{Result, anyhow};
use csv::Writer;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{self};
use std::time::Duration;
use url::Url;

#[derive(Debug, Serialize)]
struct OutputRow {
    coin: String,
    original_url: String,
    api_url: String,
    start_date: String,
    end_date: String,
    pps_profit_sum: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("==================================");
    println!("ViaBTC Multi-Coin PPS Aggregator");
    println!("==================================");

    let (start_date, end_date) = read_date_range();

    let content = fs::read_to_string("input.txt")?;

    let urls: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if urls.is_empty() {
        println!("Ссылки (URL) не найдены");
        wait_exit();
        return Ok(());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0")
        .build()?;

    let mut writer = Writer::from_path("output.csv")?;

    // -----------------------------
    // COINS CONFIG (EASILY EXTENDABLE)
    // -----------------------------
    let coins = vec!["BTC", "LTC", "KAS"];

    // store totals per coin
    let mut totals: HashMap<String, f64> = HashMap::new();

    for coin in &coins {
        totals.insert(coin.to_string(), 0.0);
    }

    for url in urls {
        println!("\nОбрабатывается ссылка:\n{url}");

        for coin in &coins {
            match process_url(
                &client,
                &url,
                coin,
                start_date.as_deref(),
                end_date.as_deref(),
            )
            .await
            {
                Ok(row) => {
                    println!("{} | {}", coin, row.pps_profit_sum);

                    *totals.get_mut(&coin.to_string()).unwrap() += row.pps_profit_sum;

                    writer.serialize(row)?;
                }
                Err(e) => {
                    println!("ОШИБКА ({coin}): {e}");
                }
            }
        }
    }

    println!("\n==============================");
    println!("ИТОГО");
    println!("==============================");

    for (coin, total) in totals {
        println!("{coin}: {total}");
    }

    writer.flush()?;

    println!("\nКонец.");
    wait_exit();

    Ok(())
}

async fn process_url(
    client: &Client,
    original_url: &str,
    coin: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<OutputRow> {
    let access_key = extract_access_key(original_url)?;

    let mut api_url = format!(
        "https://www.viabtc.com/res/observer/profit/detail?page=1&limit=200&access_key={}&user_id&utc=false&coin={}&method=summary",
        access_key, coin
    );

    if let (Some(start), Some(end)) = (start_date, end_date) {
        api_url = format!(
            "https://www.viabtc.com/res/observer/profit/detail?page=1&limit=200&start_date={}&end_date={}&access_key={}&user_id&utc=false&coin={}&method=summary",
            start, end, access_key, coin
        );
    }

    let resp = client.get(&api_url).send().await?;

    let text = resp.text().await?;

    // -----------------------------
    // DEBUG API OUTPUT
    // -----------------------------
    // fs::write("debug_last_response.json", &text)?;

    let json: Value = serde_json::from_str(&text)?;

    let items = json
        .get("data")
        .and_then(|d| d.get("data"))
        .and_then(|d| d.as_array())
        .ok_or_else(|| anyhow!("Invalid JSON structure"))?;

    let mut total = 0.0;

    for item in items {
        if let Some(v) = item.get("pps_profit") {
            if let Some(s) = v.as_str() {
                total += s.replace(",", "").parse::<f64>().unwrap_or(0.0);
            }
        }
    }

    Ok(OutputRow {
        coin: coin.to_string(),
        original_url: original_url.to_string(),
        api_url,
        start_date: start_date.unwrap_or("ALL").to_string(),
        end_date: end_date.unwrap_or("ALL").to_string(),
        pps_profit_sum: total,
    })
}

fn read_date_range() -> (Option<String>, Option<String>) {
    println!("Диапазон дат (ENTER = Полная история PPS)");

    let start = read_line("Дата начала диапазона (YYYY-MM-DD): ");
    let end = read_line("Дата окончания диапазона (YYYY-MM-DD): ");

    let start = if start.is_empty() { None } else { Some(start) };
    let end = if end.is_empty() { None } else { Some(end) };

    (start, end)
}

fn read_line(prompt: &str) -> String {
    use std::io::Write;

    print!("{prompt}");
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

fn extract_access_key(url: &str) -> Result<String> {
    let parsed = Url::parse(url)?;

    for (k, v) in parsed.query_pairs() {
        if k == "access_key" {
            return Ok(v.to_string());
        }
    }

    Err(anyhow!("access_key not found"))
}

fn wait_exit() {
    println!("\nНажмите ENTER чтобы выйти...");
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
}
