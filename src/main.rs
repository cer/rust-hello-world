#![feature(proc_macro_hygiene, decl_macro)]

// #[macro_use]
use rocket::*;

// Don't need that

//extern crate mysql;

// use rocket::State;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyResponse {
    message : String
}

struct AStruct (i32, String);

#[get("/")]
fn index(pool: State<mysql::Pool>) -> String {
    let ids = get_ids(pool);

    let response = make_response(ids);

    let x = serde_json::to_string(&response);

    x.unwrap()
}

fn make_response(ids: Vec<i32>) -> MyResponse {
    let mut s = "Hello, world!: ".to_string();
    let t  = format!("{:?}", ids);
    s.push_str(&t);
    MyResponse { message: t }
}

fn get_ids(pool: State<mysql::Pool>) -> Vec<i32> {
    let exec_result = pool.prep_exec("select id from eventuate.foo", ());
    let ids: Vec<i32> = exec_result
        .map(|result| {
            result.map(|x| x.unwrap())
                .map(|row| {
                    mysql::from_row(row)
                }).collect()
        }).unwrap();
    ids
}

#[get("/foo")]
fn foo() -> &'static str {
    "Hello, foo!"
}

fn main() {
    let pool = mysql::Pool::new("mysql://root:rootpassword@localhost:3306").unwrap();

    rocket::ignite()
        .mount("/", routes![index, foo])
        .mount("/baz", routes![foo])
        .manage(pool)
        .launch();
}
