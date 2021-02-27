# API Specification

## Note
> Supported `Content-Type` for all endpoints is limited to `application/json`.
> All `POST` / `PUT` requests must contain Request header `Content-Type: application/json`

## Resources

### /config
> #### PUT /config: updates configuration (e.g. `table_count`)

Request format (example)
```json
{
    "table_count": 10
}
```

Response format (example)

```json
{
  "status": "Ok"
}
```

### /menus
> #### POST /menus : creates menu item

Request format (example)

```json
{
    "status": "active",
    "name": "Hors-d'oeuvre / Appetiser",
    "preparation_time": 10
}
```

Response format (example)

```json
{
  "id": "94034913-b16d-40ed-82eb-1e524be7402e",
  "status": "Created"
}
```

> #### GET /menus : fetches all menu items

Response format (example)

```json
{
  "count": 7,
  "menus": [
    {
      "id": "5f8ce181-89c6-4243-8f7e-94a47e76f6ad",
      "name": "Boissons / Beverage",
      "preparation_time": 5,
      "status": "active"
    },
    {
      "id": "e690c436-8c95-43d5-96e2-5e5c5403aff5",
      "name": "Hors-d'oeuvre / Appetiser",
      "preparation_time": 10,
      "status": "active"
    },
    {
      "id": "6e171031-ffd8-4437-adfc-d70fbb91e8e0",
      "name": "Potage / Soup",
      "preparation_time": 10,
      "status": "active"
    },
    {
      "id": "682c84ff-63f6-4b4f-aa3a-2099d1932805",
      "name": "Poisson / Fish",
      "preparation_time": 15,
      "status": "active"
    },
    {
      "id": "9aa1c33e-41b3-465b-8953-e2d13f776bed",
      "name": "Entree / Entree",
      "preparation_time": 15,
      "status": "active"
    },
    {
      "id": "f2adbde4-df65-4c33-ab44-4361fd062914",
      "name": "Releves / Joints",
      "preparation_time": 15,
      "status": "active"
    },
    {
      "id": "c858198b-b9a0-4022-b71f-12e2eb5a0ee4",
      "name": "Entremets / Sweets",
      "preparation_time": 10,
      "status": "active"
    }
  ],
  "status": "Ok"
}
```

> #### DELETE /menus/:id : deletes a menu item specified by `id`
Response format (example)

```json
{
  "status": "Accepted"
}
```


### /orders

> #### POST /orders : creates an order

Request format (example)

```json
{
  "table_id": "1",
  "menu_id": "<a valid menu_id from GET /menus response>",
  "quantity": 1
}
```

Response format (example)

```json
{
  "id": "59ee00ea-ed64-429f-adf1-1d2832ced412",
  "status": "Created"
}
```

> #### PUT /orders : creates an order

Request format (example)

```json
{
  "table_id": "<table_id of the original resource>",
  "menu_id": "<menu_id of the original resource>",
  "quantity": 1,
  "state": "CANCELLED"
}
```
Response format (example)

```json
{
  "id": "59ee00ea-ed64-429f-adf1-1d2832ced412",
  "status": "Ok"
}
```

> Possible values of `state` are `ORDERED` (which is set by default while creating orders), 
> `SERVED` (means the order has been served; updating `state` to `SERVED` also updates `served_time`),
> `CANCELLED` (means the order has been cancelled), `COOKING` (means the order is under preparation). <br/>
> 
> Note: Orders with `state` other than `ORDERED` cannot be cancelled.

> #### GET /orders : fetches all orders

Response format (example)
```json
{
    "count": 2,
    "orders": [
        {
            "create_time": "Sun, 28 Feb 2021 05:11:54 +0900",
            "estimated_serve_time": "Sun, 28 Feb 2021 05:26:54 +0900",
            "id": "fb4eca17-48b6-4df9-9494-7949e2187e48",
            "menu_id": "682c84ff-63f6-4b4f-aa3a-2099d1932805",
            "menu_name": "Poisson / Fish",
            "quantity": 1,
            "served_time": null,
            "state": "ORDERED",
            "table_id": "40",
            "update_time": null
        },
        {
            "create_time": "Sun, 28 Feb 2021 05:11:54 +0900",
            "estimated_serve_time": "Sun, 28 Feb 2021 05:16:54 +0900",
            "id": "b25ab1de-7d2b-4cba-835c-2c41312b09cf",
            "menu_id": "5f8ce181-89c6-4243-8f7e-94a47e76f6ad",
            "menu_name": "Boissons / Beverage",
            "quantity": 1,
            "served_time": null,
            "state": "ORDERED",
            "table_id": "77",
            "update_time": null
        }
    ],
    "status": "Ok"
}
```

> Results can be filtered using following query parameters. <br/>
> `?table_id=<string>`, `?menu_id=<string>`, `?state=<"ORDERED"|"SERVED"|"CANCELLED"|"COOKING">` <br/>
> Use `&` to combine multiple query parameters. (e.g. `?table_id="44"&state="ORDERED"`)

> #### GET /orders/:id : fetches details of one order specified by `id`

Response format (example)

```json
{
    "order": {
        "create_time": "Sun, 28 Feb 2021 05:24:32 +0900",
        "estimated_serve_time": "Sun, 28 Feb 2021 05:34:32 +0900",
        "id": "fb1d6e9e-1e6b-44be-bb76-56eba39d97a0",
        "menu_id": "e690c436-8c95-43d5-96e2-5e5c5403aff5",
        "menu_name": "Hors-d'oeuvre / Appetiser",
        "quantity": 1,
        "served_time": null,
        "state": "ORDERED",
        "table_id": "2",
        "update_time": null
    },
    "status": "Ok"
}
```

> #### DELETE /orders/:id : deletes an order specified by `id`

Response format (example)

```json
{
    "id": "fb1d6e9e-1e6b-44be-bb76-56eba39d97a0",
    "status": "Accepted"
}
```