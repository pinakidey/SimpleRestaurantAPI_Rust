//! Application entrypoint. Runs Rocket, loads menu and runs worker threads.

#![feature(proc_macro_hygiene, decl_macro, thread_id_value, intra_doc_pointers)]
#![allow(non_snake_case)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
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
    println!("Reading Menus from File...");
    let file = File::open(Path::new("src/menu.json")).expect("File not found");
    let menus: Vec<Menu> = serde_json::from_reader(file)
        .expect("Error reading/parsing file.");

    // Use reqwest client to make remote API call
    let client = reqwest::Client::new();
    for menu in menus.iter() {
        client.post("http://localhost:8000/menus")
            .json(&menu)
            .send()
            .await?;
    }
    println!("ALL MENUS LOADED!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Spawn a thread to launch the API so that it doesn't block the main thread.
    let t = thread::spawn(move || {
        routes::rocket().launch();
    });

    // Load menu data.
    load_menu().await.expect("FAILED TO LOAD INITIAL DATA. SOME FEATURES MAY NOT WORK.");

    // To disable worker client, comment out the line below
    worker::run().await;

    t.join().expect("Couldn't join on the associated thread");

    Ok(())
}



