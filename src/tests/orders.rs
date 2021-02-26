//! Tests for functions implementing the /orders API
mod orders {
    extern crate serde_json;

    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use serde::{Deserialize, Serialize};

    use crate::rocket;
    use crate::routes::rocket;

    const VALID_MENU_PAYLOAD: &str =
        "{
            \"status\": \"active\",
            \"name\": \"Hors-d'oeuvre / Appetiser\",
            \"preparation_time\": 10
        }";
    const VALID_CREATE_ORDER_PAYLOAD_1: &str =
        "{
            \"table_id\": \"1\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
        }";
    const VALID_CREATE_ORDER_PAYLOAD_2: &str =
        "{
            \"table_id\": \"2\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
        }";
    const VALID_UPDATE_ORDER_PAYLOAD_1: &str =
        "{
            \"table_id\": \"1\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 2
        }";
    const VALID_UPDATE_ORDER_PAYLOAD_2: &str =
        "{
            \"table_id\": \"1\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 2,
            \"state\": \"SERVED\"
        }";
    const INVALID_UPDATE_ORDER_PAYLOAD_1: &str =
        "{
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 2
        }";
    const INVALID_UPDATE_ORDER_PAYLOAD_2: &str =
        "{
            \"table_id\": \"1\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 2,
            \"state\": \"SERVED\"
        }";
    const INVALID_CREATE_ORDER_PAYLOAD_1: &str =
        "{
            \"table_id\": \"1\",
            \"quantity\": 1
        }";
    const INVALID_CREATE_ORDER_PAYLOAD_2: &str =
        "{
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
        }";
    const INVALID_CREATE_ORDER_PAYLOAD_3: &str =
        "{
            \"table_id\": \"1\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\"
        }";
    const INVALID_CREATE_ORDER_PAYLOAD_4: &str =
        "{
            \"table_id\": \"101\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
        }";
    const INVALID_CREATE_ORDER_PAYLOAD_5: &str =
        "{
            \"table_id\": \"0\",
            \"menu_id\": \"MENU_ID_PLACEHOLDER\",
            \"quantity\": 1
        }";


    #[derive(Serialize, Deserialize, Debug)]
    struct Response {
        id: String,
        status: String,
    }

    #[test]
    /// Test /orders API
    fn test_orders() {
        let client = Client::new(rocket()).unwrap();
        // We have to create a menu item before creating any orders.
        let mut res = client.post("/menus")
            .header(ContentType::JSON)
            .body(VALID_MENU_PAYLOAD)
            .dispatch();
        let parsed_res: Response = serde_json::from_str(res.body_string().unwrap().as_str()).unwrap();
        let menu_id = parsed_res.id.clone();
        assert!(!menu_id.is_empty());

        /* Test create_order() */

        // Positive tests
        // Tests create_order with proper payload
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(VALID_CREATE_ORDER_PAYLOAD_1.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::Created);
        let parsed_res: Response = serde_json::from_str(res.body_string().unwrap().as_str()).unwrap();
        let order_id = parsed_res.id.clone();
        assert!(!order_id.is_empty());
        println!("{}", order_id);


        // Negative Tests
        // Tests with payload without `menu_id`
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(INVALID_CREATE_ORDER_PAYLOAD_1)
            .dispatch();
        println!("{:#?}", res.body_string());
        assert_eq!(res.status(), Status::UnprocessableEntity);

        // Tests with payload without `table_id`
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(INVALID_CREATE_ORDER_PAYLOAD_2)
            .dispatch();
        println!("{:#?}", res.body_string());
        assert_eq!(res.status(), Status::UnprocessableEntity);

        // Tests with payload without `quantity`
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(INVALID_CREATE_ORDER_PAYLOAD_3)
            .dispatch();
        println!("{:#?}", res.body_string());
        assert_eq!(res.status(), Status::UnprocessableEntity);

        // Test with invalid menu_id
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(VALID_CREATE_ORDER_PAYLOAD_1.replace("MENU_ID_PLACEHOLDER", "INVALID"))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);

        // Tests with invalid (out of range) table_id
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(INVALID_CREATE_ORDER_PAYLOAD_4.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(INVALID_CREATE_ORDER_PAYLOAD_5.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);

        /* Test get_order() */

        // Get order by id and check if order_id is present in the response body
        res = client.get(format!("/orders/{}", order_id)).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains(&order_id));

        /* Test get_orders() */

        // Get all orders and check if order_id is present in the response body
        res = client.get("/orders").header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains(&order_id));

        // Filter orders by table_id
        // We will create another order with different table to verify filtering
        res = client.post("/orders")
            .header(ContentType::JSON)
            .body(VALID_CREATE_ORDER_PAYLOAD_2.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::Created);

        // GET all orders
        res = client.get("/orders").header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains("\"count\":2"));

        // Now GET with query-param
        res = client.get("/orders?table_id=2").header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains("\"count\":1"));

        // Filter with valid menu_id
        res = client.get(format!("/orders?menu_id={}", menu_id)).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains("\"count\":2"));

        // Filter with invalid menu_id
        res = client.get(format!("/orders?menu_id={}", "INVALID")).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains("\"count\":0"));

        /* Test update_order() */

        // Update quantity
        res = client.put(format!("/orders/{}", order_id))
            .header(ContentType::JSON)
            .body(VALID_UPDATE_ORDER_PAYLOAD_1.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains(&order_id));

        // Update state
        res = client.put(format!("/orders/{}", order_id))
            .header(ContentType::JSON)
            .body(VALID_UPDATE_ORDER_PAYLOAD_2.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains(&order_id));

        // Filter with state
        res = client.get(format!("/orders?state={}", "SERVED")).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(body.contains("\"count\":1"));

        // Verify served_time is not null
        res = client.get(format!("/orders/{}", order_id)).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:#?}", body);
        assert!(!body.contains("\"served_time\":null"));

        // Negative tests
        // Try updating with invalid menu_id
        res = client.put(format!("/orders/{}", order_id))
            .header(ContentType::JSON)
            .body(VALID_UPDATE_ORDER_PAYLOAD_2.replace("MENU_ID_PLACEHOLDER", "INVALID"))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);

        // Try updating without table_id
        res = client.put(format!("/orders/{}", order_id))
            .header(ContentType::JSON)
            .body(INVALID_UPDATE_ORDER_PAYLOAD_1.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::UnprocessableEntity);

        // Try updating with a different table_id
        res = client.put(format!("/orders/{}", order_id))
            .header(ContentType::JSON)
            .body(INVALID_UPDATE_ORDER_PAYLOAD_2.replace("MENU_ID_PLACEHOLDER", menu_id.as_str()))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);
    }
}
