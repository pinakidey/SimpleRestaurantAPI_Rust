use rocket::{Request, response, Response};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8};

#[derive(Serialize, Deserialize, Debug)]
/// Model for individual order. All non-optional properties are mandatory in API Request.
// Note: Ideally, we should make quantity un-editable to maintain consistency in order.
// Order that gets created will go to a message queue for chefs to view & cook. Changing quantity
// on such order is not good. Client should create a new order for the extra items.
// However, in this project we allow updating quantity.
pub(crate) struct Order {
    pub(crate) id: Option<String>,
    pub(crate) table_id: String,
    pub(crate) menu_id: String,
    pub(crate) menu_name: Option<String>,
    pub(crate) quantity: u8,                               // updatable
    pub(crate) state: Option<String>,                      // updatable
    pub(crate) create_time: Option<String>,
    pub(crate) update_time: Option<String>,                // updatable
    pub(crate) estimated_serve_time: Option<String>,       // updatable
    pub(crate) served_time: Option<String>,                // updatable
}

#[derive(Serialize, Deserialize, Debug)]
/// Model for individual menu item
pub(crate) struct Menu {
    pub(crate) id: Option<String>,
    pub(crate) status: String,
    pub(crate) name: String,
    pub(crate) preparation_time: u8,
}

/// A thread-safe struct to save Table count
// Ideally there should be `struct Table` with various fields like `id:<String>, state: <reserved|occupied|vacant|maintenance>` etc.
// But, those are not necessary for this project.
// We assume that tables are numbered sequentially (ie. if there are 10 tables numbered 1 to 10,
// setting TableCount to 5 will re-number all tables as 1 to 5).
// Restaurant admin should not reduce TableCount when the restaurant is in business-hours.
// When creating orders, /orders API just checks that the `table_id` is <= TableCount (i.e. in range)
pub(crate) struct TableCount(pub AtomicU8);

#[derive(Serialize, Deserialize, Debug)]
/// Model for configuration
pub(crate) struct Config {
    pub(crate) table_count: u8,
}

/// An enum for order states
pub(crate) enum OrderStates {
    ORDERED,
    CANCELLED,
    COOKING,
    SERVED,
}

/// Implementation for OrderStates
impl OrderStates {
    pub fn as_str(&self) -> &'static str {
        match *self {
            OrderStates::ORDERED => "ORDERED",
            OrderStates::CANCELLED => "CANCELLED",
            OrderStates::COOKING => "COOKING",
            OrderStates::SERVED => "SERVED"
        }
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
    pub fn get_as_array() -> Vec<String> {
        vec![OrderStates::ORDERED.to_string(), OrderStates::CANCELLED.to_string(),
             OrderStates::COOKING.to_string(), OrderStates::SERVED.to_string()]
    }
}

/// Custom `ApiResponse` struct is used to set appropriate status code. Implements `Responder`
#[derive(Debug)]
pub(crate) struct ApiResponse {
    pub(crate) json: JsonValue,
    pub(crate) status: Status,
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


#[derive(FromForm)]
/// Struct for QueryParams supported by `GET /orders`
pub(crate) struct OrderQueryParams {
    pub(crate) table_id: Option<String>,
    pub(crate) menu_id: Option<String>,
    pub(crate) state: Option<String>
}
