#![feature(proc_macro_hygiene, decl_macro)]

use rocket::*;

// #[macro_use]
use mysql::{Error};

use rocket_contrib::json::Json;

use serde::{Serialize, Deserialize};

mod customer_service;
mod customer_events;
mod event_publishing;

#[derive(Serialize, Deserialize)]
struct CreateCustomerRequest {
    name : String,
    credit_limit: i64
}

#[derive(Serialize, Deserialize)]
struct CreateCustomerResponse {
    id : i64
}

impl Drop for CreateCustomerResponse {
    fn drop(&mut self) {
        println!("goodbye")
    }
}

use crate::customer_service::save_customer;
use std::ops::Deref;

#[post("/customers", format = "application/json", data = "<request>")]
fn create_customer(pool: State<mysql::Pool>, request : Json<CreateCustomerRequest>) -> Result<Json<CreateCustomerResponse>, Error> {
    let p : &mysql::Pool = pool.deref();

    let id = save_customer(p, &request.name, request.credit_limit)?;

    Ok(Json(CreateCustomerResponse { id: id }))
}


fn main() {
    let pool = mysql::Pool::new_manual(1, 1, "mysql://root:rootpassword@localhost:3306").unwrap();

    rocket::ignite()
        .mount("/", routes![create_customer])
        .manage(pool)
        .launch();
}
