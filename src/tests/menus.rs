//! Tests for functions implementing the /menus API
mod menus {
    extern crate serde_json;
    use crate::rocket;
    use rocket::local::{Client};
    use rocket::http::{Status, ContentType};
    use serde::{Deserialize, Serialize};
    use crate::routes::{rocket};

    const VALID_MENU_PAYLOAD: &str = "{
    \"status\": \"active\",
    \"name\": \"Hors-d'oeuvre / Appetiser\",
    \"preparation_time\": 10
}";
    const INVALID_MENU_PAYLOAD_1: &str = "{
    \"status\": \"active\",
    \"preparation_time\": 10
}";
    const INVALID_MENU_PAYLOAD_2: &str = "{
    \"status\": \"active\",
    \"name\": \"Hors-d'oeuvre / Appetiser\"
}";

    #[derive(Serialize, Deserialize, Debug)]
    struct Response {
        id: String,
        status: String
    }

    #[test]
    /// Test add_menu()
    fn test_add_menu() {
        let client = Client::new(rocket()).unwrap();

        // Positive tests
        // Tests add_menu with proper payload
        let mut res = client.post("/menus")
            .header(ContentType::JSON)
            .body(VALID_MENU_PAYLOAD)
            .dispatch();
        println!("{:?}", res.body_string());
        assert_eq!(res.status(), Status::Created);

        // Negative Tests
        // Tests with payload without `name`
        res = client.post("/menus")
            .header(ContentType::JSON)
            .body(INVALID_MENU_PAYLOAD_1)
            .dispatch();
        println!("{:?}", res.body_string());
        assert_eq!(res.status(), Status::UnprocessableEntity);

        // Tests with payload without `preparation_time`
        res = client.post("/menus")
            .header(ContentType::JSON)
            .body(INVALID_MENU_PAYLOAD_2)
            .dispatch();
        println!("{:?}", res.body_string());
        assert_eq!(res.status(), Status::UnprocessableEntity);
    }

    #[test]
    /// Tests get_menus()
    fn test_get_menus() {
        let client = Client::new(rocket()).unwrap();

        // Tests initial state
        let mut res = client.get("/menus").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:?}", body);
        assert!(body.contains("\"menus\":[]"));

        // Add a menu item
        let mut post_res = client.post("/menus")
            .header(ContentType::JSON)
            .body(VALID_MENU_PAYLOAD)
            .dispatch();
        assert_eq!(post_res.status(), Status::Created);

        // Get the id of the resource created
        let parsed_res: Response = serde_json::from_str(post_res.body_string().unwrap().as_str()).unwrap();
        println!("Created resource: id = {:?}", parsed_res.id);

        // Get all menus and check id is present in the response body
        res = client.get("/menus").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:?}", body);
        assert!(body.contains(&parsed_res.id));

        // Delete the previously created menu item
        res = client.delete(format!("/menus/{}", &parsed_res.id)).dispatch();
        assert_eq!(res.status(), Status::Accepted);

        // Verify that the item got deleted successfully
        res = client.get("/menus").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.body_string().unwrap();
        println!("{:?}", body);
        assert!(!body.contains(&parsed_res.id));
    }

    #[test]
    /// Tests for delete_menu()
    fn test_delete_menu() {
        // Positive test is performed inside test_get_menus()

        // Negative test
        let client = Client::new(rocket()).unwrap();
        let mut res = client.delete(format!("/menus/{}", 1)).dispatch();
        println!("{:?}", res.body_string().unwrap());
        assert_eq!(res.status(), Status::NotFound);
    }
}
