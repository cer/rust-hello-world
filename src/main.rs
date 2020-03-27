#![feature(proc_macro_hygiene, decl_macro)]

use rocket::*;

use mysql::{Error};

use rocket_contrib::json::Json;

use serde::{Serialize, Deserialize};
use hello_world::customer_service::CustomerService;
use std::sync::Arc;
use hello_world::event_publishing::DomainEventPublisher;



#[derive(Serialize, Deserialize)]
struct CreateCustomerRequest {
    name : String,
    credit_limit: i64
}

#[derive(Serialize, Deserialize)]
struct CreateCustomerResponse {
    id : i64
}

#[derive(Serialize, Deserialize)]
struct GetCustomerResponse {
    id : i64,
    name : String,
    credit_limit: i64
}

#[post("/customers", format = "application/json", data = "<request>")]
fn create_customer(customer_service : State<CustomerService>,
                   request : Json<CreateCustomerRequest>) -> Result<Json<CreateCustomerResponse>, Error> {
    let id = customer_service.save_customer(&request.name, request.credit_limit)?;

    Ok(Json(CreateCustomerResponse { id: id }))
}

#[get("/customers/<id>", format = "application/json")]
fn get_customer(customer_service : State<CustomerService>,
                   id : i64) -> Result<Option<Json<GetCustomerResponse>>, Error> {
    let maybe_customer = customer_service.find_customer(id)?;
    Ok(maybe_customer.map(|customer| {
        Json(GetCustomerResponse { id: customer.id, name: customer.name, credit_limit: customer.credit_limit })
    }))
}

fn main() {
    let pool = Arc::new(mysql::Pool::new_manual(1, 1, "mysql://root:rootpassword@localhost:3306").unwrap());
    let domain_event_publisher  = Arc::new(DomainEventPublisher{});

    let customer_service : CustomerService = CustomerService::new(&domain_event_publisher, &pool);
    // example of multi-uses of pool, etc.
    let _customer_service2 : CustomerService = CustomerService::new(&domain_event_publisher, &pool);

    rocket::ignite()
        .mount("/", routes![create_customer, get_customer])
        .manage(customer_service)
        .launch();
}
