use serde::{Serialize, Deserialize};


#[derive(Serialize)]
#[derive(Deserialize)]
struct Order {
    orderId: u64,
    tableId: u16,
    menuId: u8,
    menuItemId: u8,
    quantity: u8,
    state: String, // oneOf(ordered|cancelled|cooking|served)
}