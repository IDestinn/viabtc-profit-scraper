use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct InputRow {
    pub name: String,
    pub link: String,
}

#[derive(Debug)]
pub struct ReportRow {
    pub client_name: String,
    pub original_url: String,
    pub coin_values: HashMap<String, f64>,
    pub total_per_link: f64,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub data: ApiData,
}

#[derive(Debug, Deserialize)]
pub struct ApiData {
    pub data: Vec<ApiItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApiItem {
    pub pps_profit: String,
}
