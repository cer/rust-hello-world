use mysql::{params, Error};

use mysql::Transaction;
use mysql::IsolationLevel;

use crate::customer_events::CustomerCreatedEvent;
use crate::event_publishing::publish_event;

pub fn save_customer(pool : &mysql::Pool, name : &String, credit_limit : i64) -> Result<i64, Error> {

    let mut con = pool.get_conn().unwrap();

    let mut txn = con.start_transaction(true, Some(IsolationLevel::RepeatableRead), Some(false)).unwrap();

    let insert_result =
        txn.prep_exec("insert into eventuate.customers(name, credit_limit) values(:name, :credit_limit)",
                      params!{"name" =>  name, "credit_limit" => credit_limit});
    insert_result?;

    let id = get_last_insert_id(&mut txn)?;

    publish_event(&mut txn, "Customer".to_string(), id,
                  &CustomerCreatedEvent { name: name.clone(), credit_limit: credit_limit })?;


    txn.commit()?;

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
