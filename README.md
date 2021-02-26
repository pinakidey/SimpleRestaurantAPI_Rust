# Preface

I started learning Rust last week. I finished a course on Udemy on Rust (3-4days), 
read the rust-docs (in part) and Rocket docs (in full).

This is my first project in Rust. 
I found Rust very interesting, it made me think a lot more about WHY, at every step, instead of HOW and reminds me of C++ days.

The concepts of trait implementation, lifetime, ownership, borrowing(!), Arc/Rc/RfCell/Mutex etc. are still `volatile` in my memory, 
but I have tried my best to put up something (with sufficient tests), in a limited time.

## Assumptions
- Since the problem statement doesn't mention data needs to be stored `on-disk` persistently, 
  I am assuming an on-memory db or data-structure is fine.


## Documentation
Please check the API_SPEC.md for API specification.

## Build & Run

### Install Rust nightly for Async support and set default

`rustup toolchain install nightly` <br/>
`rustup default nightly`

### Build/Test/Run
(Inside project root directory)
`cargo build`
`cargo test`
`cargo run`



## Implementation

### Functional features


### Non-functional features
- Error-handling
- Documentation: Docs page opens @ root path
- Request format validation

### Things not implemented / limitation
- Any type of security features
- API only supports `Content-Type: application/json`
- Cross-resource validation (e.g. POST /orders wouldn't validate if `Order{menu_id}`. This kind of checks should be implemented in the business-logic layer.) 
- While creating resources, although using UUID as `id`,
  have not implemented checks for rare but possible id-collision.
  When using a real DB such occurrences will throw an error from the DB-layer
  and API-layer can retry with a new UUID.
  
### Scope of Improvements
- Use a Database to persist data.
- Use Diesel for ORM.
- Add authentication.
