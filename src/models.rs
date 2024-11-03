//! Models from the binance API
use serde::Deserialize;
use serde_this_or_that::as_f64;

#[derive(Deserialize, Debug)]
pub struct StreamEvent {
    #[serde(alias = "e")]
    pub event_type: String,
    #[serde(alias = "E")]
    pub event_time: usize,
    #[serde(alias = "s")]
    pub symbol: String,
    #[serde(alias = "U")]
    pub first_update_id: usize,
    #[serde(alias = "u")]
    pub final_update_id: usize,
    #[serde(alias = "b")]
    pub bids: Vec<Item>,
    #[serde(alias = "a")]
    pub asks: Vec<Item>,
}

#[derive(Deserialize, Debug)]
pub struct Item {
    #[serde(deserialize_with = "as_f64")]
    pub price: f64,
    #[serde(deserialize_with = "as_f64")]
    pub quantity: f64,
}

#[derive(Deserialize, Debug)]
pub struct Snapshot {
    #[serde(alias = "lastUpdateId")]
    pub last_update_id: usize,
    pub bids: Vec<Item>,
    pub asks: Vec<Item>,
}
