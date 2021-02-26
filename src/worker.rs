//! A worker/client which makes multi-threaded calls to the business logic functions (which in turn calls the APIs)

use std::thread;
use crate::logic;
use reqwest::Client;
use std::time::Duration;

pub(crate) async fn run() {
    let client: Client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build().unwrap();

    let handle = thread::spawn(move || async move {
        for i in 1..=10 {
            println!("Running Thread#{}...", i);
            let result = logic::fetch_menus(client.clone()).await;
            result.expect("Failed to fetch menu.");
            thread::sleep(Duration::from_millis(1000));
        }
    });
    handle.join().unwrap().await;
}