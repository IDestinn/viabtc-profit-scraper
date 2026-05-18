mod api;
mod config;
mod excel;
mod input;
mod models;
mod totals;
mod utils;

use anyhow::Result;
use chrono::Local;
use reqwest::Client;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;
use std::time::Duration;
use tracing::{error, info};

use api::fetch_coin_profit;
use config::*;
use excel::generate_excel_report;
use input::load_input_file;
use totals::Totals;
use utils::{read_date_range, wait_exit};

#[tokio::main]
async fn main() {
    init_panic_handler();

    if let Err(err) = run().await {
        log_error(&format!("{:#}", err));

        println!();
        println!("==================================");
        println!("КРИТИЧЕСКАЯ ОШИБКА");
        println!("==================================");
        println!("{:#}", err);
        println!();
        println!("Подробности сохранены в log.txt");

        wait_exit();
    }
}

async fn run() -> Result<()> {
    init_logger();

    println!("==================================");
    println!("ViaBTC Multi-Coin PPS Aggregator");
    println!("==================================");

    // ----------------------------------------
    // INPUT
    // ----------------------------------------
    let (start_date, end_date) = read_date_range();
    let input_file = find_input_file()?;

    println!("Файл ввода: {}", input_file);

    let clients = load_input_file(&input_file)?;

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

fn init_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let message = match panic_info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => "Unknown panic",
        };

        let location = if let Some(location) = panic_info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown".to_string()
        };

        let full_message = format!("PANIC\nLocation: {}\nMessage: {}\n", location, message);

        log_error(&full_message);

        println!();
        println!("==================================");
        println!("ПРОГРАММА АВАРИЙНО ЗАВЕРШИЛАСЬ");
        println!("==================================");
        println!("{}", full_message);
        println!("Подробности сохранены в log.txt");

        wait_exit();
    }));
}

fn log_error(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    let log_line = format!("[{}]\n{}\n\n", timestamp, message);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("log.txt") {
        let _ = file.write_all(log_line.as_bytes());
    }
}

fn find_input_file() -> Result<String> {
    use std::path::Path;

    for file in INPUT_FILES {
        if Path::new(file).exists() {
            return Ok(file.to_string());
        }
    }

    anyhow::bail!("Не найден input файл (csv/xlsx)")
}
