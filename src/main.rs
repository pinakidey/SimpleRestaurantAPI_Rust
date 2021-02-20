#![feature(proc_macro_hygiene, decl_macro)]
#![allow(non_snake_case)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

use std::{thread, time};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Duration};
use chrono::prelude::*;
use rocket::{Request, response, Response};
use rocket::fairing::AdHoc;
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests;

// In-memory storage for all resources, used as ManagedState.
type OrderMap = Mutex<HashMap<String, Order>>;
type MenuMap = Mutex<HashMap<String, Menu>>;

#[derive(Serialize, Deserialize, Debug)]
/// Model for individual order. All non-optional properties are mandatory in API Request.
struct Order {
    id: Option<String>,
    table_id: String,
    menu_id: String, //we'll not have the menu_name in Order, since that'll affect normalization
    quantity: u8,
    state: Option<String>,
    create_time: Option<String>,
    update_time: Option<String>,
    estimated_serve_time: String,
    served_time: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
/// Model for individual menu item
struct Menu {
    id: Option<String>,
    status: String,
    name: String,
    preparation_time: u8,
}

/// An enum for order states
enum OrderStates {
    ORDERED, CANCELLED, COOKING, SERVED
}

impl OrderStates {
    pub fn as_str(&self) -> &'static str {
        match *self {
            OrderStates::ORDERED => "ORDERED",
            OrderStates::CANCELLED => "CANCELLED",
            OrderStates::COOKING => "COOKING",
            OrderStates::SERVED => "SERVED"
        }
    }
}

/// Custom `ApiResponse` struct is used to set appropriate status code. Implements `Responder`
#[derive(Debug)]
struct ApiResponse {
    json: JsonValue,
    status: Status,
}

// Implement `Responder` for `ApiResponse`
impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

/// Route to POST a Menu item
#[post("/menus", format = "json", data = "<payload>")]
fn add_menu(payload: Json<Menu>, map: State<MenuMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map lock.");

    let ref id = payload.0.id;
    match id {
        None => {
            let id = Uuid::new_v4().to_string();
            let menu = Menu { id: Some(id.clone()), ..payload.0 };
            hashmap.insert(id.clone(), menu);
            ApiResponse {
                status: Status::Created,
                json: json!({
                    "status": "Created",
                    "id": id
                })
            }
        }
        Some(_id) => {
            ApiResponse {
                status: Status::BadRequest,
                json: json!({
                    "status": "BadRequest",
                    "reason": "Don't include 'id' in POST payload"
                })
            }
        }
    }
}

/// Route to GET all Menu items (we'll not implement /menus/<id> as there is no current requirement to GET a single menu item)
#[get("/menus")]
fn get_menus(map: State<MenuMap>) -> ApiResponse {
    let hashmap = map.lock().unwrap();
    let mut menus: Vec<&Menu> = Vec::new();
    for (_, menu) in hashmap.iter() {
        menus.push(menu);
    }
    ApiResponse {
        status: Status::Ok,
        json: json!({
                    "status": "Ok",
                    "menus": menus
                }),
    }
}

/// Route to DELETE a Menu item
// Ideally, menu item can't be deleted when there is an order on that item that are yet to be served.
// All endpoints under /menus should require an elevated authorization level (say, STORE_MANAGER)
// to add/delete menu items, and such operation (specially, delete) would be done in maintenance period.
#[delete("/menus/<id>")]
fn delete_menu(id: String, map: State<MenuMap>) -> ApiResponse {
    let mut hashmap = map.lock().unwrap();
    if hashmap.contains_key(&id) {
        hashmap.remove(&id);
        ApiResponse {
            status: Status::Accepted,
            json: json!({
                    "status": "Accepted"
                })
        }
    } else {
        ApiResponse {
            status: Status::NotFound,
            json: json!({
                    "status": "NotFound",
                    "reason": "Resource was not found"
                })
        }
    }
}

/// Route to create an Order
#[post("/orders", format = "json", data = "<payload>")]
fn create_order(payload: Json<Order>, map: State<OrderMap>, menus: State<MenuMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map lock.");
    let menuMap = menus.lock().expect("map lock.");
    let id = payload.0.id;
    match id {
        None => {
            // Check if the menu_id is valid
            let menu = menuMap.get(&payload.0.menu_id);
            match menu {
                None => {
                    ApiResponse {
                        status: Status::BadRequest,
                        json: json!({
                        "status": "BadRequest",
                        "reason": "Invalid menu selection."
                    })
                    }
                }
                Some(menu) => {
                    let id = Uuid::new_v4().to_string();
                    let local: DateTime<Local> = Local::now();
                    let order = Order {
                        id: Some(id.clone()),
                        state: Some(OrderStates::ORDERED.as_str().to_string()),
                        create_time: Some(local.to_rfc2822()),
                        estimated_serve_time: (local + Duration::minutes(i64::from(menu.preparation_time))).to_rfc2822(),
                        ..payload.0 };
                    hashmap.insert(id.clone(), order);
                    ApiResponse {
                        status: Status::Created,
                        json: json!({
                        "status": "Created",
                        "id": id
                    })
                    }}
            }
        }
        Some(_) => {
            ApiResponse {
                status: Status::BadRequest,
                json: json!({
                    "status": "BadRequest",
                    "reason": "Don't include 'id' in POST payload"
                })
            }
        }
    }
}

/// Route to update an Order
#[put("/orders/<id>", format = "json", data = "<payload>")]
fn update_order(id: String, payload: Json<Order>, map: State<OrderMap>) -> Option<JsonValue> {
    let mut hashmap = map.lock().unwrap();
    if hashmap.contains_key(&id) {
        let local: DateTime<Local> = Local::now();
        let order = Order {
            id: Some(id.clone()),
            update_time: Some(local.to_rfc2822()),
            ..payload.0 };
        hashmap.insert(id, order);
        Some(json!({
            "status": "ok"
            })
        )
    } else {
        None
    }
}

/// Route to get an Order
#[get("/orders/<id>")]
fn get_order(id: String, map: State<OrderMap>) -> Option<JsonValue> {
    let hashmap = map.lock().unwrap();
    let order = hashmap.get(&id);
    match order {
        None => { None }
        Some(order) => {
            Some(json!({
                "status": "ok",
                "order": order
            }))
        }
    }
}

#[catch(404)]
fn not_found() -> ApiResponse {
    ApiResponse {
        status: Status::NotFound,
        json: json!({
            "status": "NotFound",
            "reason": "Resource was not found"
        }),
    }
}

#[catch(422)]
fn unprocessable() -> ApiResponse {
    ApiResponse {
        status: Status::UnprocessableEntity,
        json: json!({
            "status": "UnprocessableEntity",
            "reason": "Invalid request-body format. Please check API spec."
        }),
    }
}

#[catch(500)]
fn server_error() -> JsonValue {
    json!({
        "status": "error",
        "reason": "There was an error at the server side."
    })
}

pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![
            add_menu, get_menus, delete_menu,
            create_order, update_order, get_order
        ])
        .register(catchers![not_found, unprocessable, server_error])
        .manage(Mutex::new(HashMap::<String, Order>::new()))
        .manage(Mutex::new(HashMap::<String, Menu>::new()))
        .attach(AdHoc::on_launch("Launch Printer", |_| {
            println!("Rocket is about to launch!");
        }))
}

/// Loads initial menus into managed state via API
async fn load_menu() -> Result<(), Box<dyn Error>> {
    println!("Reading File...");
    let file = File::open(Path::new("src/menu.json")).expect("File not found");
    let menus: Vec<Menu> = serde_json::from_reader(file)
        .expect("Error reading/parsing file.");
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
    //rocket().launch();
    // Spawn a thread to launch the API so that it doesn't block the main thread.
    let t = thread::spawn(move || {
        rocket().launch();
    });

    // Load menu data.
    let result = load_menu().await;
    result.expect("FAILED TO LOAD INITIAL DATA. SOME FEATURES MAY NOT WORK.");

    t.join().unwrap();
    Ok(())
}

