
use serde::Serialize;
use std::time::SystemTime;
use std::collections::HashMap;
use sqlx::*;

pub struct DomainEventPublisher {

}

impl DomainEventPublisher  {

pub async fn publish_event<T : Serialize<>, I : ToString>(&self, txn : &mut Transaction<'_, MySql>, aggregate_type : String, aggregate_id : I, event : &T) -> Result<()> {
    let now = SystemTime::now();

    // Find better way

    let t = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
    let id = t.to_string();
    let creation_time = t.to_string();
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

    sqlx::query("insert into eventuate.message(id, destination, headers, payload, creation_time) values(?,?,?,?,?)")
        .bind(&id)
        .bind(aggregate_type)
        .bind(headers)
        .bind(payload)
        .bind(creation_time)
        .execute(txn)
        .await?;
    Ok(())

}
}
