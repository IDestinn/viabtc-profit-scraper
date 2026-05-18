use anyhow::{Result, anyhow};
use calamine::{Reader, open_workbook_auto};
use std::path::Path;

use crate::models::InputRow;

pub fn load_input_file(path: &str) -> Result<Vec<InputRow>> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(anyhow!("Файл {} не найден", path));
    }

    let extension = path_obj
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "csv" => load_csv(path),
        "xlsx" | "xlsm" | "xls" => load_excel(path),
        _ => Err(anyhow!("Неподдерживаемый формат файла: {}", extension)),
    }
}

fn load_csv(path: &str) -> Result<Vec<InputRow>> {
    let mut rows = Vec::new();

    let content = std::fs::read_to_string(path)?;

    for (index, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        // skip header
        if index == 0 && line.to_lowercase().contains("name") {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ';').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Некорректная строка CSV: {}", line));
        }

        rows.push(InputRow {
            name: parts[0].trim().to_string(),
            link: parts[1].trim().to_string(),
        });
    }

    Ok(rows)
}

fn load_excel(path: &str) -> Result<Vec<InputRow>> {
    let mut workbook = open_workbook_auto(path)?;

    let sheet_names = workbook.sheet_names().to_owned();

    if sheet_names.is_empty() {
        return Err(anyhow!("Excel файл не содержит листов"));
    }

    let first_sheet = &sheet_names[0];

    let range = workbook.worksheet_range(first_sheet)?;

    let mut rows = Vec::new();

    for (index, row) in range.rows().enumerate() {
        // skip header
        if index == 0 {
            continue;
        }

        if row.len() < 2 {
            continue;
        }

        let name = row[0].to_string().trim().to_string();
        let link = row[1].to_string().trim().to_string();

        if name.is_empty() || link.is_empty() {
            continue;
        }

        rows.push(InputRow { name, link });
    }

    Ok(rows)
}
