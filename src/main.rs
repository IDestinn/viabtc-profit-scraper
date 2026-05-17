mod api;
mod config;
mod excel;
mod input;
mod models;
mod totals;
mod utils;

use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info};

use api::fetch_coin_profit;
use config::*;
use excel::generate_excel_report;
use input::load_input_file;
use totals::Totals;
use utils::{read_date_range, wait_exit};

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    println!("==================================");
    println!("ViaBTC Multi-Coin PPS Aggregator");
    println!("==================================");

    // ----------------------------------------
    // INPUT
    // ----------------------------------------
    let (start_date, end_date) = read_date_range();
    let clients = load_input_file(INPUT_FILE)?;

    if clients.is_empty() {
        println!("input.csv пуст");
        wait_exit();
        return Ok(());
    }

    // ----------------------------------------
    // HTTP CLIENT
    // ----------------------------------------
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .user_agent(USER_AGENT)
        .build()?;

    // ----------------------------------------
    // REPORT DATA
    // ----------------------------------------
    let mut report_rows = Vec::new();

    let mut totals = Totals::new();

    // ----------------------------------------
    // PROCESS
    // ----------------------------------------
    for client_row in &clients {
        info!("Обрботка пользователя {}", client_row.name);

        let mut link_total = 0.0;

        let mut coin_results: HashMap<String, f64> = HashMap::new();

        for coin in COINS {
            match fetch_coin_profit(
                &client,
                &client_row.link,
                coin,
                start_date.as_deref(),
                end_date.as_deref(),
            )
            .await
            {
                Ok(value) => {
                    info!("{} {} => {}", client_row.name, coin, value);

                    coin_results.insert(coin.to_string(), value);

                    link_total += value;

                    totals.add_coin_total(coin, value);
                    totals.add_grand_total(value);
                }

                Err(e) => {
                    error!("ОШИБКА ОБРАБОТКИ {} {}: {}", client_row.name, coin, e);

                    coin_results.insert(coin.to_string(), 0.0);
                }
            }
        }

        report_rows.push(models::ReportRow {
            client_name: client_row.name.clone(),
            original_url: client_row.link.clone(),
            coin_values: coin_results,
            total_per_link: link_total,
        });
    }

    // ----------------------------------------
    // EXCEL
    // ----------------------------------------
    generate_excel_report(OUTPUT_FILE, &report_rows, &totals, &start_date, &end_date)?;

    println!();
    println!("==================================");
    println!("ГОТОВО");
    println!("==================================");
    println!("Файл сохранён: {}", OUTPUT_FILE);

    wait_exit();

    Ok(())
}

fn init_logger() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
}
