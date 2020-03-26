use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CustomerCreatedEvent {
    pub name : String,
    pub credit_limit: i64
}
