This package is for giving utils for web servers supporting the following web frameworks:

- actix
- rocket
- axum
- hyper

These will help the developer use these tools outside of the box to have uniform runtime states, config handles, and 
error handling for api endpoints. This will enable developers to fuse servers together to call endpoints via memory.

## Error handling

This package has errors that can be imported with the following statement:

```rust
use nanoservices_utils::errors::{NanoServiceError, NanoServiceErrorStatus};
```

The `NanoServiceError` is the error struct that handles the message and the error status of the error. To construct an error, you can use the following:

```rust
let error = NanoServiceError::new(
    "could not find the item".to_string(),
    NanoServiceErrorStatus::NotFound,
)
```

The `NanoServiceErrorStatus` will convert to a HTTP response code that corresponds with the message. Without any feature selection, the error will just be an error. However, if you select one or more of the following features:

- axum
- actix
- rocket
- hyper

The error will be able to be converted to that framework's HTTP response. This means that your library can return `NanoServiceError` structs for errors and these errors will be able to convert into HTTP responses for those webframeworks. To see how this works, lets look at the following example where we have a function that will not allow a number more than 10 with the following code:

```rust
use nanoservice_utils::errors::{NanoServiceError, NanoServiceErrorStatus};

pub fn check(number: i32) -> Result<i32, NanoServiceError> {
    if number > 10 {
        return Err(NanoServiceError::new(
            "number is too large".to_string(),
            NanoServiceErrorStatus::BadRequest,
        ));
    }
    Ok(number)
}
```

We can then call this function and exploit the `?` operator to return a HTTP response automatically if an error is thrown with the following frameworks:

### Actix

```rust
use actix_web::{HttpResponse, web};
use nanoservices_utils::errors::NanoServiceError;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct Number {
    number: i32,
}


pub async fn some_api_endpoint(body: web::Json<Number>) -> Result<HttpResponse, NanoServiceError> {
    let number = check(body.number)?;
    // can do something else here
    Ok(HttpResponse::Ok().json(number))
}
```

### Axum

```rust
use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
};
use nanoservices_utils::errors::NanoServiceError;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct Number {
    number: i32,
}

pub async fn some_api_endpoint(Json(body): Json<Number>) -> Result<impl IntoResponse, NanoServiceError> {
    let number = check(body.number)?;
    // can do something else here
    Ok((StatusCode::OK, axum::Json(number)))
}
```

### Rocket

```rust
use rocket::serde::json::Json;
use nanoservices_utils::errors::NanoServiceError;
use rocket::http::Status;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]  // Specify that Rocket's `serde` should be used
pub struct Number {
    number: i32,
}

#[post("/some/api/endpoint", format = "json", data = "<body>")]
pub async fn some_api_endpoint(body: Json<Number>) -> Result<Json<i32>, (Status, String)> {
    let number = check(body.number)?;
    // can do something else here
    Ok(Json(number))
}
```

## Beta Utils

I'm currently supporting the following utils:

- JWT
- Config
- error handling for server endpoints
- runtime state

but these are not polished