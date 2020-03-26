
use mysql::{params, Error};

use mysql::Transaction;

use serde::Serialize;
use std::time::SystemTime;
use std::collections::HashMap;

pub struct DomainEventPublisher {

}

impl DomainEventPublisher  {

pub fn publish_event<T : Serialize<>, I : ToString>(&self, txn : &mut Transaction, aggregate_type : String, aggregate_id : I, event : &T) -> Result<(), Error> {
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
}
