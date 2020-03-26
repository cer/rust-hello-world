#![feature(proc_macro_hygiene, decl_macro)]

// #[macro_use]
use rocket::*;

// #[macro_use]
use mysql::{params, Error};

use mysql::Transaction;
use mysql::IsolationLevel;

// Don't need that

//extern crate mysql;

// use rocket::State;

use rocket_contrib::json::Json;

use serde::{Serialize, Deserialize};
use std::time::SystemTime;
use std::collections::HashMap;

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

#[derive(Serialize, Deserialize)]
struct CustomerCreatedEvent {
    name : String,
    credit_limit: i64
}

#[post("/customers", format = "application/json", data = "<request>")]
fn create_customer(pool: State<mysql::Pool>, request : Json<CreateCustomerRequest>) -> Result<Json<CreateCustomerResponse>, Error> {
    let mut con = pool.get_conn().unwrap();

    let mut txn = con.start_transaction(true, Some(IsolationLevel::RepeatableRead), Some(false)).unwrap();

    let id = save_customer(&mut txn, &request.name, request.credit_limit)?;

    publish_event(&mut txn, "Customer".to_string(), id,
                  &CustomerCreatedEvent { name: request.name.clone(), credit_limit: request.credit_limit })?;

    txn.commit()?;

    Ok(Json(CreateCustomerResponse { id: id }))
}

fn publish_event<T : Serialize<>, I : ToString>(txn : &mut Transaction, aggregate_type : String, aggregate_id : I, event : &T) -> Result<(), Error> {
    let now = SystemTime::now();

    let id = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let creation_time = id;
    let payload = serde_json::to_string(event).unwrap();

    let mut header_values : HashMap<&str, String> = HashMap::new();


    /*
    header_values.insert("ID", String::from(id));

    header_values.insert(String::from("PARTITION_ID"), String::from("Y"));
    header_values.insert(String::from("DESTINATION"), String::from("Y"));
    header_values.insert(String::from("DATE"), String::from("Y"));

    header_values.insert(String::from("event-type"), String::from("Y"));
    header_values.insert(String::from("event-aggregate-type"), String::from("Y"));
    */

    header_values.insert("event-aggregate-id", aggregate_id.to_string());

    let headers = serde_json::to_string(&header_values).unwrap();

    dbg!(&headers);

    let insert_result =
        txn.prep_exec("insert into eventuate.message(id, destination, headers, payload, creation_time) values(:id, :destination, :headers, :payload, :creation_time)",
                      params!{"id" =>  id,
                      "destination" => aggregate_type,
                      "headers" => headers,
                      "payload" => payload,
                      "creation_time" => creation_time,
                      });
    insert_result?;

    Ok(())

}

fn save_customer(txn : &mut Transaction, name : &String, credit_limit : i64) -> Result<i64, Error> {

    let insert_result =
        txn.prep_exec("insert into eventuate.customers(name, credit_limit) values(:name, :credit_limit)",
                       params!{"name" =>  name, "credit_limit" => credit_limit});
    insert_result?;

    let id = get_last_insert_id(txn)?;

    Ok(id)

}

fn get_last_insert_id(txn : &mut Transaction) -> Result<i64, Error> {
    let exec_result = txn.prep_exec("SELECT LAST_INSERT_ID()", ());
    let ids: Vec<i64> = exec_result
        .map(|result| {
            result.map(|x| x.unwrap())
                .map(|row| {
                    mysql::from_row(row)
                }).collect()
        })?;
    let id = ids.first().unwrap();
    Ok(*id)
}

fn main() {
    let pool = mysql::Pool::new_manual(1, 1, "mysql://root:rootpassword@localhost:3306").unwrap();

    rocket::ignite()
        .mount("/", routes![create_customer])
        .manage(pool)
        .launch();
}
