//! A worker/client which uses multiple threads to call the business logic functions (which in turn call the APIs)
//! This file is used just for simulating a set of clients, and thus have no tests for itself or much of error handling.

use std::thread;

use chrono::{DateTime, Local};
use rand::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::logic::*;
use crate::models::{Menu, Menus, Orders};

const THREAD_COUNT: usize = 2;
const TABLE_COUNT: u8 = 100;

const CREATE_ORDER_PAYLOAD: &str =
    "{
            \"table_id\": \"TABLE_ID_PLACEHOLDER\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
    }";
const SET_CONFIG_PAYLOAD: &str =
    "{
            \"table_count\": TABLE_COUNT_PLACEHOLDER
    }";

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    id: String,
    status: String,
}

pub(crate) async fn run() {
    async fn execute() {
        let client: Client = Client::new();

        // Set table_count to 10
        let result = set_table_count(&client, SET_CONFIG_PAYLOAD.replace("TABLE_COUNT_PLACEHOLDER", TABLE_COUNT.to_string().as_str())).await.expect("Failed to set table count.");
        let parsedResult: Value = serde_json::from_str(result.as_str()).expect("Failed to parse result.");
        println!("Set table_count to {}. => {}", TABLE_COUNT, parsedResult);

        // Fetch menus
        let result = fetch_menus(&client).await.expect("Failed to fetch menu.");
        // Parse
        let menusJson: Menus = serde_json::from_str(result.as_str()).expect("Failed to parse menu.");
        // Save menus array
        let menus: Vec<Menu> = menusJson.menus;

        // Create orders
        // Make random number (count) of orders with items selected from the menu, with random table_id & quantity=1 for each
        let count = rand::thread_rng().gen_range(1..=menus.len());
        let table = rand::thread_rng().gen_range(1..=TABLE_COUNT);
        println!("Creating {} orders...", count);
        let mut orders: Vec<String> = Vec::new();
        for idx in 1..=count {
            println!("ORDER#{}", idx);
            let order = CREATE_ORDER_PAYLOAD.replace("TABLE_ID_PLACEHOLDER", table.to_string().as_str())
                .replace("MENU_ID_PLACEHOLDER", &menus.get(idx - 1).unwrap().id.as_ref().unwrap().as_str());

            let result = create_order(&client, order).await.expect("Failed to create orders.");
            let parsedResult: Response = serde_json::from_str(result.as_str()).expect("Failed to parse result.");
            println!("Created order#{}", parsedResult.id);
            orders.push(parsedResult.id);
        }
        println!("{:?}", orders);

        // Get all orders by table_id
        let result = get_orders_by_table(&client, table.to_string()).await.expect("Failed to get orders by table.");
        let parsedResult: Orders = serde_json::from_str(result.as_str()).expect("Failed to parse result.");
        println!("All Orders from Table{} # {:#?}", table.to_string(), parsedResult);

        // Get details of last order and then Cancel the order
        match orders.pop() {
            None => {
                println!("No orders have been crated yet.")
            }
            Some(order_id) => {
                let result = get_order(&client, order_id.to_string()).await.expect("Failed to get order details.");
                let parsedResult: Value = serde_json::from_str(result.as_str()).expect("Failed to parse result.");
                println!("Last order#{:?}", parsedResult);

                let result = delete_order(&client, order_id.to_string()).await.expect("Failed to delete order.");
                let parsedResult: Response = serde_json::from_str(result.as_str()).expect("Failed to parse result.");
                println!("Deleted order#{}", parsedResult.id);
            }
        }

        // Get remaining orders (i.e. state: ORDERED)
        let result = get_remaining_orders(&client, table.to_string()).await.expect("Failed to get remaining orders.");
        let parsedResult: Orders = serde_json::from_str(result.as_str()).expect("Failed to parse result.");
        println!("Remaining orders# {:#?}", parsedResult);

        // Print name and time-to-serve of remaining items
        parsedResult.orders.iter()
            .for_each(|order| {
                println!("Menu: {}, Time to serve: {}min",
                         order.menu_name.as_ref().unwrap(),
                         (DateTime::parse_from_rfc2822(order.estimated_serve_time.as_ref().unwrap().as_str()).unwrap().naive_local() - Local::now().naive_local()).num_minutes())
            });
    }

    let handles = (1..=THREAD_COUNT)
        .into_iter()
        .map(|i| {
            println!("Submitted Thread {}", i);
            thread::spawn(move || {execute()})
        })
        .collect::<Vec<_>>();
    println!("Waiting ...");
    for handle in handles {
        handle.join().unwrap().await;
    }
}