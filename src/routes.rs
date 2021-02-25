//! This file contains functions implementing the API routes,
//! and their helper functions and Rocket instance generator.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Duration};
use chrono::prelude::*;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::LenientForm;
use rocket::{State};
use rocket_contrib::json::Json;
use uuid::Uuid;

use crate::models::{ApiResponse, Menu, Order, OrderQueryParams, OrderStates};

// In-memory storage for all resources, used as ManagedState.
type OrderMap = Mutex<HashMap<String, Order>>;
type MenuMap = Mutex<HashMap<String, Menu>>;


/// Route to POST a Menu item
#[post("/menus", format = "json", data = "<payload>")]
fn add_menu(payload: Json<Menu>, map: State<MenuMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map lock.");
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
    let hashmap = map.lock().unwrap();
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
    let mut hashmap = map.lock().unwrap();
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
fn create_order(payload: Json<Order>, map: State<OrderMap>, menus: State<MenuMap>) -> ApiResponse {
    let mut hashmap = map.lock().expect("map lock.");
    let menuMap = menus.lock().expect("map lock.");

    // Check if the menu_id is valid
    let menu = menuMap.get(&payload.0.menu_id);
    match menu {
        None => {generateBadRequestResponse("Invalid menu selection.")}
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
            hashmap.insert(id.clone(), order);
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
    let mut hashmap = map.lock().unwrap();
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
                )
            }
            // If menu_id is different, reject. table_id can be changed (as if customer has moved to another table)
            if payload.menu_id.ne(&order.menu_id) {
                return generateBadRequestResponse("Invalid menu_id.")
            }

            let local: DateTime<Local> = Local::now();
            let updatedOrder = Order {
                id: Some(id.clone()),
                //overwrite the unchangeable fields using original order field values
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
    let hashmap = map.lock().unwrap();
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
/// `table_id=<string>`, `menu_id=<string>`
#[get("/orders?<params..>")]
fn get_orders(params: Option<LenientForm<OrderQueryParams>>, map: State<OrderMap>) -> ApiResponse {
    let hashmap = map.lock().unwrap();
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

// For consistency, Orders should not be deleted. They can be marked CANCELLED.

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
            add_menu, get_menus, delete_menu,
            create_order, update_order, get_order, get_orders
        ])
        .register(catchers![bad_request, not_found, unprocessable, server_error])
        .manage(Mutex::new(HashMap::<String, Order>::new()))
        .manage(Mutex::new(HashMap::<String, Menu>::new()))
        .attach(AdHoc::on_launch("Launch Printer", |_| {
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
            isMatch
        }
        None => { true }
    }
}