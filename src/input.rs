use anyhow::{Result, anyhow};
use std::fs;

use crate::models::InputRow;

pub fn load_input_file(path: &str) -> Result<Vec<InputRow>> {
    let content = fs::read_to_string(path)?;

    let mut rows = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        // Skip header
        if index == 0 && line.to_lowercase().contains("name") {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ';').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Некорректная строка: {}", line));
        }

        rows.push(InputRow {
            name: parts[0].trim().to_string(),
            link: parts[1].trim().to_string(),
        });
    }

    Ok(rows)
}
