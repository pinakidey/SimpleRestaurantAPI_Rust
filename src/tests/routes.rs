
mod routes {
    use crate::rocket;
    use crate::routes::{rocket};
    use rocket::local::Client;
    use std::any::{Any, TypeId};

    #[test]
    /// Tests for router()
    fn test_router() {
        let client = Client::new(rocket()).expect("Valid Rocket");
        assert_eq!(client.type_id(), TypeId::of::<Client>());
    }
}