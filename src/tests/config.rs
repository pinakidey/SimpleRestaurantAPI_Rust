//! Tests for functions implementing the /config API
mod config {
    extern crate serde_json;

    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use serde::{Deserialize, Serialize};

    use crate::rocket;
    use crate::routes::rocket;

    const VALID_CONFIG_PAYLOAD: &str =
        "{
            \"table_count\": 10
        }";
    const INVALID_CONFIG_PAYLOAD: &str =
        "{
            \"table\": 10
        }";

    #[derive(Serialize, Deserialize, Debug)]
    struct Response {
        status: String,
    }

    #[test]
    /// Test /config API
    fn test_config() {
        let client = Client::new(rocket()).unwrap();
        let res = client.put("/config")
            .header(ContentType::JSON)
            .body(VALID_CONFIG_PAYLOAD)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let res = client.put("/config")
            .header(ContentType::JSON)
            .body(INVALID_CONFIG_PAYLOAD)
            .dispatch();
        assert_eq!(res.status(), Status::UnprocessableEntity);
    }
}