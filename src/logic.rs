//! This file contains functions implementing the business logics
//! as given here https://github.com/paidy/interview/blob/master/SimpleRestaurantApi.md

use std::error::Error;
use reqwest::Client;

const TABLE_COUNT : u8 = 100;

// Preparation
// Get all menus and save menu_id
/// Get all menus
pub(crate) async fn fetch_menus(client: Client) -> Result<(), Box<dyn Error>> {
    let res = client.get("http://localhost:8000/menus")
        .send()
        .await?
        .text()
        .await?;
    println!("{:#?}", res);
    Ok(())
}

// The client (the restaurant staff “devices” making the requests) MUST be able to:
// add one or more items with a table number, remove an item for a table,
// and query the items still remaining for a table.













// The application MUST, upon creation request, store the item, the table number, and how long the item will take to cook.
// The application MUST, upon deletion request, remove a specified item for a specified table number.
// The application MUST, upon query request, show all items for a specified table number.
// The application MUST, upon query request, show a specified item for a specified table number.
// The application MUST accept at least 10 simultaneous incoming add/remove/query requests.
// The client MAY limit the number of specific tables in its requests to a finite set (at least 100).
// The application MAY assign a length of time for the item to prepare as a random time between 5-15 minutes.
// The application MAY keep the length of time for the item to prepare static (in other words, the time does not have to be counted down in real time, only upon item creation and then removed with the item upon item deletion).
