#![feature(proc_macro_hygiene, decl_macro)]

use actix_web::{get, post, web, App, HttpResponse, HttpServer};

use serde::{Serialize, Deserialize};
use hello_world::customer_service::CustomerService;
use std::sync::Arc;
use hello_world::event_publishing::DomainEventPublisher;
use hello_world::mysql_util::new_connection_pool;

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

#[post("/customers")]
async fn create_customer(customer_service : web::Data<CustomerService>,
                   request : web::Json<CreateCustomerRequest>) -> actix_web::Result<HttpResponse> {
    let id = customer_service.save_customer(&request.name, request.credit_limit).await.unwrap();
    Ok(HttpResponse::Ok().json(CreateCustomerResponse { id: id }))
}

#[get("/customers/{id}")]
async fn get_customer(customer_service : web::Data<CustomerService>,
                path : web::Path<(i64,)>) -> actix_web::Result<HttpResponse> {
    let x = customer_service.find_customer(path.into_inner().0).await.unwrap();
    match x {
        Some(customer) => Ok(HttpResponse::Ok().json(GetCustomerResponse { id: customer.id, name: customer.name, credit_limit: customer.credit_limit})),
        None => Ok(HttpResponse::NotFound().body("found found")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = new_connection_pool().await.unwrap();
    let domain_event_publisher  = Arc::new(DomainEventPublisher{});
    let customer_service : CustomerService = CustomerService::new(&domain_event_publisher, &pool);
    let customer_service_data= web::Data::new(customer_service);

    HttpServer::new(move || {
        App::new()
            .app_data(customer_service_data.clone())
            // .data_factory(|| {
            //     let result : Result<CustomerService, ()> = Ok(CustomerService::new(domain_event_publisher.clone(), &pool));
            //     future::ready(result) })
            .service(create_customer)
            .service(get_customer)
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
