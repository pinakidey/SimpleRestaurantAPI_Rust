//! This file contains functions implementing the business logics
//! as given [here](<https://github.com/paidy/interview/blob/master/SimpleRestaurantApi.md>)

use std::error::Error;
use reqwest::Client;
use url::Url;

// Preparation
/// Get all menus
pub(crate) async fn fetch_menus(client: &Client) -> Result<String, Box<dyn Error>> {
    let res = client.get("http://localhost:8000/menus")
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}

// The client (the restaurant staff “devices” making the requests) MUST be able to: add one or more items with a table number, remove an item for a table, and query the items still remaining for a table.
// => Use `create_order()`, `delete_order()`, `get_remaining_orders()`

// The application MUST, upon creation request, store the item, the table number, and how long the item will take to cook.
// => Use `create_order()` to store all these info plus alpha (see Order struct in models.rs). For multiple order items, call multiple times (each "order" resource can have only one item by design, for "order" to be used as message in downstream pipeline).
// Use `get_remaining_orders()` to get all remaining orders. Calculate time-to-serve using (estimated_serve_time - now) at client-side. (See worker.rs)

// The application MUST, upon deletion request, remove a specified item for a specified table number.
// => Use `get_remaining_orders()` to fetch ordered items by table. Then use `delete_order()` to CANCEL any un-served order.

// The application MUST, upon query request, show all items for a specified table number.
// => Use `get_orders_by_table()` to fetch all orders by table_id. Using filter such as `?state="ORDERED"` is advised.

// The application MUST, upon query request, show a specified item for a specified table number.
// => Use `get_orders_by_table()` to fetch all orders by table_id. Then use `get_order()` to GET an specific order.

// The application MUST accept at least 10 simultaneous incoming add/remove/query requests.
// => Change `THREAD_COUNT` in worker.rs to increase/decrease client thread count (within limitation of heap/stack size).

// The client MAY limit the number of specific tables in its requests to a finite set (at least 100).
// => Use `set_table_count()`. Check format of `Config` in models.rs.

// The application MAY assign a length of time for the item to prepare as a random time between 5-15 minutes.
// => `preparation_time` for each menu item is set while creating the menu. Check `menu.json`.

// The application MAY keep the length of time for the item to prepare static (in other words, the time does not have to be counted down in real time, only upon item creation and then removed with the item upon item deletion).
// => It's kept static. Use `get_remaining_orders()` to fetch remaining orders and Calculate time-to-serve using (estimated_serve_time - now) at client-side. (See worker.rs)



/// Create an order
pub(crate) async fn create_order(client: &Client, payload: String) -> Result<String, Box<dyn Error>> {
    let res = client.post("http://localhost:8000/orders")
        .header("Content-Type", "application/json")
        .body(payload)
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}

/// Cancel an order (update `state` to `CANCELLED`)
pub(crate) async fn delete_order(client: &Client, order_id: String) -> Result<String, Box<dyn Error>> {
    let res = client.delete(Url::parse(format!("http://localhost:8000/orders/{}", order_id).as_str())?)
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}

/// Get remaining orders by table_id
pub(crate) async fn get_remaining_orders(client: &Client, table_id: String) -> Result<String, Box<dyn Error>> {
    let res = client.get(Url::parse(format!("http://localhost:8000/orders?table_id={}&state={}", table_id, "ORDERED").as_str())?)
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}

/// Get all orders by table_id
pub(crate) async fn get_orders_by_table(client: &Client, table_id: String) -> Result<String, Box<dyn Error>> {
    let res = client.get(Url::parse(format!("http://localhost:8000/orders?table_id={}", table_id).as_str())?)
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}

/// Get an order by order_id
pub(crate) async fn get_order(client: &Client, order_id: String) -> Result<String, Box<dyn Error>> {
    let res = client.get(Url::parse(format!("http://localhost:8000/orders/{}", order_id).as_str())?)
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}

/// Set table_count
pub(crate) async fn set_table_count(client: &Client, payload: String) -> Result<String, Box<dyn Error>> {
    let res = client.put("http://localhost:8000/config")
        .header("Content-Type", "application/json")
        .body(payload)
        .send()
        .await?
        .text()
        .await?;
    //println!("{:#?}", res);
    Ok(res)
}








