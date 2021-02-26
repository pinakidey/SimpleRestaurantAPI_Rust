#![feature(proc_macro_hygiene, decl_macro, const_fn)]
#![allow(non_snake_case)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::thread;

use crate::models::Menu;

#[cfg(test)]
mod tests;
mod routes;
mod models;
mod worker;
mod logic;

// See routes.rs for API implementation

/// Loads initial menus into managed state via API
async fn load_menu() -> Result<(), Box<dyn Error>> {
    println!("Reading File...");
    let file = File::open(Path::new("src/menu.json")).expect("File not found");
    let menus: Vec<Menu> = serde_json::from_reader(file)
        .expect("Error reading/parsing file.");

    // Use reqwest client to make remote API call
    let client = reqwest::Client::new();
    for menu in menus.iter() {
        println!("{:#?}", menu);
        let res = client.post("http://localhost:8000/menus")
            .json(&menu)
            .send()
            .await?;
        println!("{:?}", res.status());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Spawn a thread to launch the API so that it doesn't block the main thread.
    let t = thread::spawn(move || {
        routes::rocket().launch();
    });

    // Load menu data.
    let result = load_menu().await;
    result.expect("FAILED TO LOAD INITIAL DATA. SOME FEATURES MAY NOT WORK.");

    worker::run().await;

    t.join().expect("Couldn't join on the associated thread");

    Ok(())
}



