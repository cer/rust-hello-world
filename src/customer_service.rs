
use sqlx::mysql::*;
use sqlx::{Pool, Row};
use sqlx::Transaction;


use crate::customer_events::CustomerCreatedEvent;
use crate::event_publishing::DomainEventPublisher;

use std::sync::Arc;

pub struct CustomerService {
    domain_event_publisher: Arc<DomainEventPublisher>,
    pool: Arc<Pool<MySql>>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomerDTO {
    pub id : i64,
    pub name : String,
    pub credit_limit: i64
}

impl CustomerService {
    pub fn new(domain_event_publisher: &Arc<DomainEventPublisher>, pool: &Arc<Pool<MySql>>) -> CustomerService {
        let x : Arc<DomainEventPublisher> = domain_event_publisher.clone();
        let y : Arc<Pool<MySql>> = pool.clone();
        CustomerService{ domain_event_publisher: x, pool: y }
    }

    pub async fn save_customer(&self, name: &String, credit_limit: i64) -> Result<i64, sqlx::Error> {
        let mut txn = self.pool.begin().await?;

        sqlx::query("insert into eventuate.customers(name, credit_limit) values(?, ?)")
            .bind(name)
            .bind(credit_limit)
            .execute(&mut txn)
            .await?;

        let id = get_last_insert_id(&mut txn).await?;

        self.domain_event_publisher .publish_event(&mut txn, "Customer".to_string(), id,
                                             &CustomerCreatedEvent { name: name.clone(), credit_limit: credit_limit }).await?;

        txn.commit().await?;

        Ok(id)
    }


    pub async fn find_customer(&self, id : i64) -> Result<Option<CustomerDTO>, sqlx::Error> {
        let mut txn = self.pool.begin().await?;

        let customer = sqlx::query("SELECT name, credit_limit from eventuate.customers where id = ? ")
            .bind(id)
            .map(|row| -> Result<CustomerDTO, sqlx::Error> {
                let name = row.try_get("name")?;
                let credit_limit :i64 = row.try_get("credit_limit")?;
                Ok(CustomerDTO { id, name, credit_limit: credit_limit })
            })
            .fetch_optional(&mut txn)
            .await?;

       txn.commit().await?;

        match customer  {
            Some(r) => {
                let c = r?;
                Ok(Some(c))
            }
            None => Ok(None)
        }
    }

}

async fn get_last_insert_id(txn: &mut Transaction<'_, MySql>) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT LAST_INSERT_ID() as ID")
        .fetch_one(txn)
        .await?;
    let id: u64  = row.try_get("ID")?;
    Ok(id as i64)
}
