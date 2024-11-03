//! The top level order books module
//! Provides the OrderBooks struct which if created in an async runtime
//! will update itself forever
mod models;

use anyhow::{anyhow, Result};
use float_ord::FloatOrd;
use futures::StreamExt;
use log::{debug, info, warn};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;

use models::{Snapshot, StreamEvent};

/// Top Level Struct containing order books for a set of securities
pub struct OrderBooks {
    pub books: HashMap<String, OrderBook>,
}

impl OrderBooks {
    pub fn new(names: &[String]) -> Self {
        let mut books = HashMap::new();
        for name in names {
            let book = OrderBook::new(name.to_string());
            // For each book spawn a task to keep it updated forever
            tokio::spawn(book.clone().update_forever());
            books.insert(name.to_string(), book);
        }
        Self { books }
    }
}

/// Struct representing an OrderBook for a given security
/// - BTreeMaps are used to keep price levels ordered
/// - Arc/Mutexes are used so that the book can be updated
///   forever from a spawned task while still being readable
///   from other tasks.
#[derive(Clone, Debug)]
pub struct OrderBook {
    name: String,
    /// bids - Mapping of price levels to quantities
    /// Raw floats can't be used as keys in maps so use FloatOrd instead
    bids: Arc<Mutex<BTreeMap<FloatOrd<f64>, f64>>>,
    /// asks - Mapping of price levels to quantities
    /// Raw floats can't be used as keys in maps so use FloatOrd instead
    asks: Arc<Mutex<BTreeMap<FloatOrd<f64>, f64>>>,
}

impl OrderBook {
    pub fn new(name: String) -> Self {
        Self {
            name,
            bids: Arc::new(Mutex::new(BTreeMap::new())),
            asks: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl OrderBook {
    /// Update the order book forever, graciously handling errors
    pub async fn update_forever(self) {
        loop {
            match self.update_until_error().await {
                Ok(_) => info!("Stream closed for OrderBook {}", self.name),
                Err(e) => warn!("OrderBook {} failed with error: {}", self.name, e),
            }
        }
    }

    /// Use the binance snapshot API and websockets API to update the order book
    /// until the websocket connection closes or an error is hit
    pub async fn update_until_error(&self) -> Result<()> {
        // Clear any existing state, as it may be invalid
        self.asks.lock().unwrap().clear();
        self.bids.lock().unwrap().clear();

        // Initialise websocket stream
        let (ws_stream, _) = connect_async(self.event_stream_url()).await?;
        let (_, mut read) = ws_stream.split();

        // Wait until events start arriving on the stream
        let first_event: StreamEvent = serde_json::from_slice(
            &read
                .next()
                .await
                .ok_or_else(|| anyhow!("Failed to read stream"))??
                .into_data(),
        )?;
        let mut last_update_id = first_event.final_update_id;

        // Get a snapshot to initially populate the order book
        let snapshot = reqwest::get(self.snapshot_url())
            .await?
            .json::<Snapshot>()
            .await?;
        self.populate_from_snapshot(&snapshot);

        // While the websocket connection is open, update the order book with events from the stream.
        // An event should be sent on the connection at least once a second. To prevent stale
        // order books, error out if no new message is received in 5 seconds
        loop {
            match timeout(Duration::from_secs(5), read.next()).await {
                Err(_) => Err(anyhow!(
                    "Websocket stream received no new messages for 5 seconds"
                ))?,
                Ok(Some(message)) => {
                    let data = message?.into_data();
                    let event: StreamEvent = match serde_json::from_slice(&data) {
                        Ok(event) => {
                            debug!("Stream event {:?}", event);
                            event
                        }
                        Err(e) => {
                            info!("Unable to parse message, ignore. Error: {e}");
                            continue;
                        }
                    };

                    // Verify that no events have been missed
                    if event.first_update_id != last_update_id + 1 {
                        Err(anyhow!("Missed updates"))?
                    } else {
                        last_update_id = event.final_update_id;
                    };

                    // Update the order book
                    self.update_from_event(&event);
                }
                Ok(None) => break,
            }
        }

        Ok(())
    }

    pub fn update_from_event(&self, event: &StreamEvent) {
        // Update the bids
        for bid in event.bids.iter() {
            self.update_bid(bid.price, bid.quantity);
        }

        // Update the asks
        for ask in event.asks.iter() {
            self.update_ask(ask.price, ask.quantity)
        }
    }

    pub fn populate_from_snapshot(&self, snapshot: &Snapshot) {
        // Update the bids
        for bid in snapshot.bids.iter() {
            self.update_bid(bid.price, bid.quantity);
        }

        // Update the asks
        for ask in snapshot.asks.iter() {
            self.update_ask(ask.price, ask.quantity)
        }
    }

    pub fn update_bid(&self, price: f64, quantity: f64) {
        if quantity == 0.0 {
            self.bids.lock().unwrap().remove(&FloatOrd(price));
        } else {
            self.bids.lock().unwrap().insert(FloatOrd(price), quantity);
        }
    }

    pub fn update_ask(&self, price: f64, quantity: f64) {
        if quantity == 0.0 {
            self.asks.lock().unwrap().remove(&FloatOrd(price));
        } else {
            self.asks.lock().unwrap().insert(FloatOrd(price), quantity);
        }
    }

    pub fn event_stream_url(&self) -> String {
        format!(
            "wss://stream.binance.com:9443/ws/{}@depth",
            self.name.to_lowercase()
        )
    }

    pub fn snapshot_url(&self) -> String {
        format!("https://api.binance.com/api/v3/depth?symbol={}", self.name)
    }
}

/// Basic implementation of display displaying up to the top
/// 20 bids and asks.
impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let items_to_display = *([
            self.bids.lock().unwrap().len(),
            self.asks.lock().unwrap().len(),
            20,
        ]
        .iter()
        .min()
        .unwrap());

        let rows = self
            .bids
            .lock()
            .unwrap()
            .iter()
            .rev()
            .take(items_to_display)
            .zip(self.asks.lock().unwrap().iter().take(items_to_display))
            .map(|((bid_price, bid_quantity), (ask_price, ask_quantity))| {
                format!(
                    "{:<10} {:<10} | {:<10} {:<10}",
                    bid_quantity, bid_price.0, ask_price.0, ask_quantity
                )
            })
            .collect::<Vec<_>>();

        let output_string = self.name.clone()
            + "\r\n"
            + &format!("{:^21} | {:^21}\r\n", "Bids", "Asks")
            + &rows.join("\r\n")
            + "\r\n";
        write!(f, "{}", output_string)
    }
}
