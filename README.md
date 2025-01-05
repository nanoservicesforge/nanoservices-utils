This package is for giving utils for web servers. We have the following features:

- database traits macros for dependency injection of DB connections and mocking of DB connections in testing
- Tokio based pub sub macros for event driven programming entirely in Tokio
- error handling for web frameworks (Actix, Axum, Rocket, Hyper)
- Config handling trait for environment variables and overwrites for testing

## Data Access Layer (`DAL` feature)

If you enable the `dal` feature you currently get access to macros that massively reduce the amount of boilerplate code you need to write for your data access layer. The following is an example of how to use the DAL:

```rust
use nanoservices_utils::{define_dal_transactions, impl_transaction};

// define the schemas that are going to be used for database transactions 
// (traits for specific databases and serialization still need to be implemented)
struct NewUser {
    name: String,
}

struct User {
    id: i32,
    name: String,
}

// Construct traits and map methods to function signatures
define_dal_transactions!(
    CreateUser => create(user: NewUser) -> i32,
    GetUser => get(id: i32) -> User,
    DeleteUser => delete(id: i32) -> bool
);

// create an empty struct that we can pass as a handle through a function
struct PostgresHandle;

// implement the `CreateUser` trait for the `PostgresHandle` struct using the `create_user_postgres` function
#[impl_transaction(PostgresHandle, CreateUser, create)]
async fn create_user_postgres(user: NewUser) -> Result<i32, NanoServiceError> {
    Ok(1)
}

// test the function to see how it works
let new_user = NewUser {
    name: "John Doe".to_string(),
};
let outcome = PostgresHandle::create(new_user).await.unwrap();
assert_eq!(outcome, 1);
``` 

## Pub Sub Tokio event driven programming (`tokio-pub-sub` feature)

If you enable the `tokio-pub-sub` feature you get access to the following macros:

```rust
use nanoservices_utils::subscribe_to_event;
use nanoservices_utils::config_tokio_event_runtime;
use nanoservices_utils::publish_event;
```

In our `main.rs` file where we are defining the entry point for our `tokio` runtime we must configure the event runtime with the following code:

```rust
config_tokio_event_runtime!();
```

Everything else can be done in any file, but the `config_tokio_event_runtime` needs to be in the `main.rs` file. We can now make subscriptions to events with the following code:

```rust
#[derive(Serialize, Deserialize, Debug)]
struct One;

#[derive(Serialize, Deserialize, Debug)]
struct Two;

#[derive(Serialize, Deserialize, Debug)]
pub struct AddNumbers {
    pub num1: i32,
    pub num2: i32,
}

#[subscribe_to_event]
async fn test(one: One) {
    println!("calling from test function with: {:?}", one);
    let two = Two {};
    publish_event!(two);
}

#[subscribe_to_event]
async fn test2(two: Two) {
    println!("calling from test2 function with: {:?}", two);
}

#[subscribe_to_event]
pub async fn add_numbers(add_numbers: AddNumbers) {
    println!("Adding numbers: {:?}", add_numbers);
    let result = add_numbers.num1 + add_numbers.num2;
    println!("Result: {}", result);
    let two = Two {};
    publish_event!(two);
}
```

Here the `#[subscribe_to_event]` macro inspects the input. If the function is a subscriber then we can only have one input which is a struct that we are subscribing to. This struct needs to implement the `Serialize` and `Deserialize` traits. So, if we publish an event with the `AddNumbers` then the `add_numbers` function will be called with the `AddNumbers` struct as the input. Multiple functions can subscribe to the same struct. We can test this with the following code:

```rust
#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let one = One {};
    test(one).await;
    let two = One {};
    publish_event!(two);
    let add_ints = AddNumbers {
        num1: 1,
        num2: 2,
    };
    publish_event!(add_ints);
    std::thread::sleep(std::time::Duration::from_secs(5));
}
```

Which will give the following output:

```
Initializing function: test
Initializing function: test2
Initializing function: add_numbers
Hello, world!
calling from test function with: One
calling from test2 function with: Two
calling from test function with: One
calling from test2 function with: Two
Adding numbers: AddNumbers { num1: 1, num2: 2 }
Result: 3
calling from test2 function with: Two
```

We can see that our functions are initialized before we even see the `Hello, world!`. This is because our subscribers are registered before the `main` function is called. 

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

The error will be able to be converted to that framework's HTTP response. This means that your library can return `NanoServiceError` structs for errors and these errors will be able to convert into HTTP responses for those webframeworks. 

You can also map any expression returning a `Result` to return a `NanoServiceError` on error with the code below:

```
use nanoservices_utils::safe_eject;


let some_outcome = safe_eject!(some_function());
```

To see how error handling works for web frameworks, lets look at the following example where we have a function that will not allow a number more than 10 with the following code:

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
- runtime state

but these are not polished
