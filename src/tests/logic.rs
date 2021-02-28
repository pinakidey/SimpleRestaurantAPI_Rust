//! Tests for functions in logic.rs

mod logic {
    use reqwest::Client;

    use crate::logic::*;
    use crate::models::{Menu, Menus, Response};

    const SET_CONFIG_PAYLOAD: &str =
        "{
            \"table_count\": 10
    }";
    const CREATE_ORDER_PAYLOAD: &str =
        "{
            \"table_id\": \"1\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
    }";

    #[tokio::test]
    #[ignore]
    // This test requires Rocket to have ignited! To run this test, use `cargo test -- --ignored` after application start.
    // We can use [mockito](https://docs.rs/mockito/0.29.0/mockito/), but let's keep things simple!
    async fn test_logic_after_application_start() {
        let client: Client = Client::new();
        let menus: Vec<Menu> ;
        let firstMenuId: &str ;
        let order_id: String;

        /* Test set_table_count() */
        assert!(set_table_count(&client, SET_CONFIG_PAYLOAD.to_string()).await.unwrap().contains("Ok"));

        /* Test set_table_count() */
        match fetch_menus(&client).await {
            Ok(result) => {
                let menusJson: Menus = serde_json::from_str(result.as_str()).expect("Failed to parse menu.");
                menus = menusJson.menus;
            }
            Err(_) => {panic!("Failed to fetch menus.")}
        }
        assert!(menus.len() > 0);
        firstMenuId = &menus.first().unwrap().id.as_ref().unwrap().as_str();

        /* Test create_order */
        match create_order(&client, CREATE_ORDER_PAYLOAD.replace("MENU_ID_PLACEHOLDER", firstMenuId).to_string()).await {
            Ok(result) => {
                let response: Response = serde_json::from_str(result.as_str()).expect("Failed to response.");
                assert_eq!(response.status, "Created".to_string());
                order_id = (*response.id).parse().unwrap();
            }
            Err(_) => {panic!("Failed to create order.")}
        }

        /* Test get_remaining_orders() */
        assert!(get_remaining_orders(&client, "1".to_string()).await.unwrap().contains(firstMenuId));

        /* Test get_orders_by_table() */
        assert!(get_orders_by_table(&client, "1".to_string()).await.unwrap().contains(firstMenuId));

        /* Test get_order() */
        assert!(get_order(&client, order_id.clone()).await.unwrap().contains(firstMenuId));

        /* Test delete_order() */
        assert!(delete_order(&client, order_id.clone()).await.unwrap().contains("Accepted"));

    }
}