use chrono::NaiveDate;
use std::io::{self, Write};

pub fn read_date_range() -> (Option<String>, Option<String>) {
    println!("Диапазон дат (ENTER = вся история)");

    let start = read_line("Дата начала (YYYY-MM-DD): ");
    let end = read_line("Дата конца (YYYY-MM-DD): ");

    let start = validate_date(start);
    let end = validate_date(end);

    (start, end)
}

fn validate_date(input: String) -> Option<String> {
    if input.trim().is_empty() {
        return None;
    }

    match NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        Ok(_) => Some(input),
        Err(_) => {
            println!("Неверный формат даты: {}", input);
            None
        }
    }
}

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);

    let _ = io::stdout().flush();

    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

pub fn wait_exit() {
    println!();
    println!("Нажмите ENTER чтобы выйти...");

    let mut s = String::new();

    let _ = io::stdin().read_line(&mut s);
}
