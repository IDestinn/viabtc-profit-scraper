use std::collections::HashMap;

use crate::config::COINS;

#[derive(Debug)]
pub struct Totals {
    pub per_coin: HashMap<String, f64>,
    pub grand_total: f64,
}

impl Totals {
    pub fn new() -> Self {
        let mut per_coin = HashMap::new();

        for coin in COINS {
            per_coin.insert(coin.to_string(), 0.0);
        }

        Self {
            per_coin,
            grand_total: 0.0,
        }
    }

    pub fn add_coin_total(&mut self, coin: &str, value: f64) {
        *self.per_coin.entry(coin.to_string()).or_insert(0.0) += value;
    }

    pub fn add_grand_total(&mut self, value: f64) {
        self.grand_total += value;
    }
}
