use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::env;
use std::time::Duration;

use orderbook::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise logging
    stderrlog::new()
        .module(module_path!())
        .verbosity(log::Level::Info)
        .init()
        .unwrap();

    // Get the crypto pairs from the args
    let args: Vec<String> = env::args().collect();
    let crypto_pairs: Vec<String> = args[1].split(",").map(|x| x.to_string()).collect();

    // Create the Order Books
    let order_books = OrderBooks::new(&crypto_pairs);

    // Basic UI to display the order books
    let mut pair_index: i32 = 0;
    enable_raw_mode()?;
    loop {
        if poll(Duration::from_millis(1_000))? {
            let event = read()?;

            if event == Event::Key(KeyCode::Left.into()) {
                pair_index = (pair_index - 1).rem_euclid(crypto_pairs.len() as i32)
            } else if event == Event::Key(KeyCode::Right.into()) {
                pair_index = (pair_index + 1).rem_euclid(crypto_pairs.len() as i32)
            } else if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        }

        print!("\x1B[2J\x1B[1;1H");
        print!(
            "{}",
            order_books
                .books
                .get(&crypto_pairs[pair_index as usize].to_string())
                .unwrap()
        );
    }
    disable_raw_mode()?;

    Ok(())
}
