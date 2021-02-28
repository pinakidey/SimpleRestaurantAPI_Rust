# Preface

I started learning Rust last week. I finished a [course](https://github.com/pinakidey/certificates/blob/main/The_Rust_Programming_Language.jpg) on Udemy on Rust (3-4days), 
read the rust-docs (in part), Rocket docs (in full) and docs of various other crates as required.

This is my first project in Rust. 
I found Rust very interesting, it made me think a lot more about WHY, at every step, instead of HOW and reminds me of C++ days.

The concepts of trait implementation, lifetime, ownership, borrowing(!), Arc/Rc/RfCell/Mutex etc. are still `volatile` in my memory, 
but I have tried my best to put up something (with sufficient tests), in a limited time.

There would surely be places where snippets could be a bit more concise and 'functional' if I was more accustomed to Rust's huge repertoire of higher-order functions.
But, I'm sure to have a grasp on those in time, just like any other languages. 

## Assumptions
- Since the problem statement doesn't mention data needs to be stored `on-disk` persistently, 
  I am assuming an on-memory db, or a **thread-safe data-structure** is fine.


## Documentation
Please check the [API_SPEC.md](API_SPEC.md) for API specification.

## Build & Run

### Install Rust nightly for Async support and set default

>`rustup toolchain install nightly` <br/>
`rustup default nightly`

### Build/Test/Run
(Inside project root directory, run the following commands)
>`cargo build` <br/>
`cargo test`  <br/>
`cargo run`   <br/>
(Since, the worker threads are run on application start, there would be lots of logs being written.) <br/>
 
>(Run tests that require application to be running)<br/>
`cargo test -- --ignored` <br/>
(To create docs, run) <br/>
`cargo doc --no-deps --target-dir docs`

> Note: After `cargo run`, if there is an error similar to `"An established connection was aborted by the software in your host machine."`,
> please run the application using IDE (e.g. IntelliJ IDEA).
> <br/>To know more about this error, see [here](https://github.com/SergioBenitez/Rocket/issues/209).
> <br/>A quick solution might be setting `Rocket.toml` > `[development]` > `address` to `"127.0.0.1"`

### Test API using POSTMAN

- Load the POSTMAN collection from [here](scripts/SimpleRestaurantAPI.postman_collection.json) in your POSTMAN client using `import`. 
- Make sure API is running on `localhost:8000`.
- Send a `GET /menus` request to load menus. The postman collection uses environment variable.
- Send a `POST /orders` request to create an order.
- Then try other Requests as you like.
- `PUT /config` sets table_count. If you see `Invalid table selection.` error in `POST /orders` request, use this to set `table_count`.

### Multi-threaded Clients
`worker.rs` uses 10 threads to make parallel async API calls with randomly selected data.
The clients are started automatically at application start. Check `Terminal/IDE` logs to verify output.  

## Implementation

### Functional features
**Method mentioned below belongs to `logic.rs`**

- The client (the restaurant staff “devices” making the requests) MUST be able to: add one or more items with a table number, remove an item for a table, and query the items still remaining for a table.
> Use `create_order()`, `delete_order()`, `get_remaining_orders()`

- The application MUST, upon creation request, store the item, the table number, and how long the item will take to cook.
> Use `create_order()` to store all these info plus alpha (see Order struct in models.rs). For multiple order items, call multiple times (each "order" resource can have only one item by design, for "order" to be used as message in downstream pipeline).
Use `get_remaining_orders()` to get all remaining orders. Calculate time-to-serve using (estimated_serve_time - now) at client-side. (See worker.rs)

- The application MUST, upon deletion request, remove a specified item for a specified table number.
> Use `get_remaining_orders()` to fetch ordered items by `table_id`. Then use `delete_order()` to CANCEL any un-served order.

- The application MUST, upon query request, show all items for a specified table number.
> Use `get_orders_by_table()` to fetch all orders by `table_id`. Using filter such as `?state="ORDERED"` is advised.

- The application MUST, upon query request, show a specified item for a specified table number.
> Use `get_orders_by_table()` to fetch all orders by `table_id`. Then use `get_order()` to GET a specific order.

- The application MUST accept at least 10 simultaneous incoming add/remove/query requests.
> Change `THREAD_COUNT` in worker.rs to increase/decrease client thread count (within limitation of heap/stack size).

- The client MAY limit the number of specific tables in its requests to a finite set (at least 100).
> Use `set_table_count()`. Check format of `Config` in `models.rs`.

- The application MAY assign a length of time for the item to prepare as a random time between 5-15 minutes.
> `preparation_time` for each menu item is set while creating the menu. Check `menu.json`.

- The application MAY keep the length of time for the item to prepare static (in other words, the time does not have to be counted down in real time, only upon item creation and then removed with the item upon item deletion).
> It's kept static. Use `get_remaining_orders()` to fetch remaining orders and Calculate time-to-serve using (estimated_serve_time - now) at client-side. (See worker.rs)


### Non-functional features
- Error-handling
- Documentation: Docs pages are mounted on http://localhost:8000/simple_restaurant_api/ (if you had run `cargo doc`)
- Request format validation

### Things not implemented / limitation
- Any type of security features
- API support for `Content-Type` other than `application/json`
- While creating resources, although using UUID as `id`,
  have not implemented checks for rare but possible id-collision.
  When using a real DB such occurrences will throw an error from the DB
  and API/logic can retry with a new UUID.
  
### Scope of Improvements
- Use a Database to persist data on disk.
- Use Diesel for ORM.
- Add authentication.
- Add CORS support (using `rocket_cors` or fairing). 
