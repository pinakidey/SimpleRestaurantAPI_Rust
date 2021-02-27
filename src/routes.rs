//! This file contains functions implementing the API routes,
//! and their helper functions and Rocket instance generator.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

use chrono::{DateTime, Duration};
use chrono::prelude::*;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::LenientForm;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::{StaticFiles};
use uuid::Uuid;

use crate::models::{ApiResponse, Config, Menu, Order, OrderQueryParams, OrderStates, TableCount};

// Types used by Rocket ManagedState for in-memory storage of resources.
type OrderMap = Mutex<HashMap<String, Order>>;
type MenuMap = Mutex<HashMap<String, Menu>>;

const DEFAULT_TABLE_COUNT: u8 = 100;

/// Route to POST a Menu item
#[post("/menus", format = "json", data = "<payload>")]
fn add_menu(payload: Json<Menu>, map: State<MenuMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map locked.");
    let id = Uuid::new_v4().to_string();
    let menu = Menu { id: Some(id.clone()), ..payload.0 };
    hashmap.insert(id.clone(), menu);
    ApiResponse {
        status: Status::Created,
        json: json!({
                    "status": "Created",
                    "id": id
                }),
    }
}

/// Route to GET all Menu items (we'll not implement /menus/<id> as there is no current requirement to GET a single menu item)
#[get("/menus")]
fn get_menus(map: State<MenuMap>) -> ApiResponse {
    let hashmap = map.lock().expect("map locked.");
    let mut menus: Vec<&Menu> = Vec::new();
    for (_, menu) in hashmap.iter() {
        menus.push(menu);
    }
    ApiResponse {
        status: Status::Ok,
        json: json!({
                    "status": "Ok",
                    "count": menus.len(),
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
    let mut hashmap = map.lock().expect("map locked.");
    if hashmap.contains_key(&id) {
        hashmap.remove(&id);
        ApiResponse {
            status: Status::Accepted,
            json: json!({
                    "status": "Accepted"
                }),
        }
    } else { generateResourceNotFoundResponse() }
}

/// Route to create an Order
#[post("/orders", format = "json", data = "<payload>")]
fn create_order(payload: Json<Order>, orders: State<OrderMap>, menus: State<MenuMap>, tables: State<TableCount>) -> ApiResponse {
    let mut orderMap = orders.lock().expect("map locked.");
    let menuMap = menus.lock().expect("map locked.");

    // Check configured table_count. If table_count is 0, orders cannot be placed.
    let table_count: u8 = tables.0.load(Ordering::Relaxed);
    if table_count < 1 {
        return ApiResponse {
            status: Status::ServiceUnavailable,
            json: json!({
                        "status": "ServiceUnavailable",
                        "reason": "We are not accepting orders now. Please try again later."
                    }),
        };
    }


    // Check if table_id is valid
    match payload.0.table_id.clone() {
        val => {
            let allowed_table_id = 1..=table_count;
            if !allowed_table_id.contains(&val.parse::<u8>().unwrap_or(0)) {
                return generateBadRequestResponse("Invalid table selection.");
            }
        }
    }
    // Check if the menu_id is valid
    let menu = menuMap.get(&payload.0.menu_id);
    match menu {
        None => { generateBadRequestResponse("Invalid menu selection.") }
        Some(menu) => {
            let id = Uuid::new_v4().to_string();
            let local: DateTime<Local> = Local::now();
            let order = Order {
                id: Some(id.clone()),
                menu_name: Some(menu.name.clone()),
                state: Some(OrderStates::ORDERED.to_string()),
                create_time: Some(local.to_rfc2822()),
                estimated_serve_time: Some((local + Duration::minutes(i64::from(menu.preparation_time))).to_rfc2822()),
                ..payload.0
            };
            orderMap.insert(id.clone(), order);
            ApiResponse {
                status: Status::Created,
                json: json!({
                        "status": "Created",
                        "id": id
                    }),
            }
        }
    }
}

/// Route to update an Order
#[put("/orders/<id>", format = "json", data = "<payload>")]
fn update_order(id: String, payload: Json<Order>, map: State<OrderMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map locked.");
    let order = hashmap.get(&id);
    match order {
        None => { generateResourceNotFoundResponse() }
        Some(order) => {
            // If order has already been served/cancelled reject update request
            if [OrderStates::SERVED.to_string(), OrderStates::CANCELLED.to_string()]
                .contains(order.state.as_ref().unwrap()) {
                return generateBadRequestResponse(
                    format!("Order {} has already been {}",
                            &id, order.state.as_ref().unwrap()).as_str()
                );
            }
            // If menu_id is different, reject.
            payload.menu_id.ne(&order.menu_id).then(|| return generateBadRequestResponse("Invalid menu_id."));
            // If table_id is different, reject.
            payload.table_id.ne(&order.table_id).then(|| return generateBadRequestResponse("Invalid table_id."));

            let local: DateTime<Local> = Local::now();
            let updatedOrder = Order {
                id: Some(id.clone()),
                //overwrite the unchangeable fields using original order field values
                table_id: order.table_id.clone(),
                menu_id: order.menu_id.clone(),
                menu_name: order.menu_name.clone(),
                create_time: order.create_time.clone(),
                update_time: Some(local.to_rfc2822()),
                state: payload.state.as_ref()
                    .map(|s| OrderStates::get_as_array().contains(s))
                    .and(payload.state.clone()).or(order.state.clone()),
                served_time: payload.state.as_ref()
                    .map(|s| s.eq(&OrderStates::SERVED.to_string()))
                    .and(Some(local.to_rfc2822())),
                ..payload.0
            };
            hashmap.insert(id.clone(), updatedOrder);
            ApiResponse {
                status: Status::Ok,
                json: json!({
                        "status": "Ok",
                        "id": id
                    }),
            }
        }
    }
}

/// Route to get an Order
#[get("/orders/<id>")]
fn get_order(id: String, map: State<OrderMap>) -> ApiResponse {
    let hashmap = map.lock().expect("map locked.");
    let order = hashmap.get(&id);
    match order {
        None => { generateResourceNotFoundResponse() }
        Some(order) => {
            ApiResponse {
                status: Status::Ok,
                json: json!({
                    "status": "Ok",
                    "order": order
                }),
            }
        }
    }
}

/// Route to get all Orders. Supports the following query-params to filter result-set:
/// `table_id=<string>`, `menu_id=<string>`, `state=<"ORDERED"|"SERVED"|"CANCELLED"|"COOKING">`
#[get("/orders?<params..>")]
fn get_orders(params: Option<LenientForm<OrderQueryParams>>, map: State<OrderMap>) -> ApiResponse {
    let hashmap = map.lock().expect("map locked.");
    let orders: Vec<&Order> = hashmap.iter()
        .map(|(_, order)| order)
        .filter(|order| matchesParams(params.as_ref(), order))
        .collect();
    ApiResponse {
        status: Status::Ok,
        json: json!({
                    "status": "Ok",
                    "count": orders.len(),
                    "orders": orders
                }),
    }
}

// For consistency, Orders should not be deleted. So, delete_order() sets the order.state to CANCELLED.
#[delete("/orders/<id>")]
fn delete_order(id: String, map: State<OrderMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map locked.");
    let order = hashmap.get(&id);
    match order {
        None => { generateResourceNotFoundResponse() }
        Some(order) => {
            // Order can only be deleted if its state is ORDERED/
            if OrderStates::ORDERED.to_string().ne(order.state.as_ref().unwrap()) {
                return generateBadRequestResponse(
                    format!("Order {} has already been {}",
                            &id, order.state.as_ref().unwrap()).as_str()
                );
            }
            let local: DateTime<Local> = Local::now();
            let updatedOrder = Order {
                id: Some(id.clone()),
                //overwrite the unchangeable fields using original order field values
                table_id: order.table_id.clone(),
                menu_id: order.menu_id.clone(),
                menu_name: order.menu_name.clone(),
                create_time: order.create_time.clone(),
                update_time: Some(local.to_rfc2822()),
                state: Some(OrderStates::CANCELLED.to_string()),
                served_time: order.served_time.clone(),
                quantity: order.quantity.clone(),
                estimated_serve_time: order.estimated_serve_time.clone(),
            };
            hashmap.insert(id.clone(), updatedOrder);
            ApiResponse {
                status: Status::Accepted,
                json: json!({
                        "status": "Accepted",
                        "id": id
                    }),
            }
        }
    }
}


/// Route to update configuration (idempotent operation)
#[put("/config", format = "json", data = "<payload>")]
fn update_config(payload: Json<Config>, tables: State<TableCount>) -> ApiResponse {
    // For now, we have only one config.
    tables.inner().0.store(payload.0.table_count, Ordering::Relaxed);
    ApiResponse {
        status: Status::Ok,
        json: json!({
              "status": "Ok"
              }),
    }
}


#[catch(400)]
/// Catcher for `400: Bad Request`
fn bad_request() -> ApiResponse {
    generateBadRequestResponse("Please verify the request payload and headers.")
}

#[catch(404)]
/// Catcher for `404: Not Found`
fn not_found() -> ApiResponse { generateResourceNotFoundResponse() }

#[catch(422)]
/// Catcher for `422: Unprocessable Entity`
fn unprocessable() -> ApiResponse {
    ApiResponse {
        status: Status::UnprocessableEntity,
        json: json!({
            "status": "UnprocessableEntity",
            "reason": "Invalid request-body format. Please check API specification."
        }),
    }
}

#[catch(500)]
/// Catcher for `500: Internal Server Error`
fn server_error() -> ApiResponse {
    ApiResponse {
        status: Status::InternalServerError,
        json: json!({
            "status": "InternalServerError",
            "reason": "There was an error at the server side. Please try again later."
        }),
    }
}


/* Rocket Instance Generator */

/// Creates a Rocket instance (Builder pattern)
pub fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![
            update_config, add_menu, get_menus, delete_menu,
            create_order, update_order, get_order, get_orders, delete_order
        ])
        .mount("/", StaticFiles::from("docs/doc"))
        .register(catchers![bad_request, not_found, unprocessable, server_error])
        .manage(Mutex::new(HashMap::<String, Order>::new()))
        .manage(Mutex::new(HashMap::<String, Menu>::new()))
        .manage(TableCount(AtomicU8::new(DEFAULT_TABLE_COUNT)))
        .attach(AdHoc::on_launch("Logger", |_| {
            println!("Rocket is about to launch!");
        }))
}


/* HELPER FUNCTIONS */
/// A generator function for 400 response
fn generateBadRequestResponse(reason: &str) -> ApiResponse {
    ApiResponse {
        status: Status::BadRequest,
        json: json!({
                    "status": "BadRequest",
                    "reason": reason
                }),
    }
}

/// A generator function for 404 response
fn generateResourceNotFoundResponse() -> ApiResponse {
    ApiResponse {
        status: Status::NotFound,
        json: json!({
                    "status": "NotFound",
                    "reason": "Resource was not found"
                }),
    }
}

/// A predicate to match order properties with query params
fn matchesParams(params: Option<&LenientForm<OrderQueryParams>>, order: &Order) -> bool {
    let mut isMatch = true;
    match params {
        Some(params) => {
            if let Some(table_id) = &params.table_id {
                isMatch &= order.table_id.eq(table_id);
            }
            if let Some(menu_id) = &params.menu_id {
                isMatch &= order.menu_id.eq(menu_id);
            }
            if let Some(state) = &params.state {
                isMatch &= order.state.as_ref().unwrap_or(&"INVALID".to_string()).eq(state);
            }
            isMatch
        }
        None => { true }
    }
}
