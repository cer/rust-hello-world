
use mysql::*;
use mysql::prelude::*;

use crate::customer_events::CustomerCreatedEvent;
use crate::event_publishing::DomainEventPublisher;

use std::sync::Arc;

pub struct CustomerService {
    domain_event_publisher: Arc<DomainEventPublisher>,
    pool: Arc<mysql::Pool>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomerDTO {
    pub id : i64,
    pub name : String,
    pub credit_limit: i64
}

impl CustomerService {
    pub fn new(domain_event_publisher: &Arc<DomainEventPublisher>, pool: &Arc<mysql::Pool>) -> CustomerService {
        let x : Arc<DomainEventPublisher> = domain_event_publisher.clone();
        let y : Arc<mysql::Pool> = pool.clone();
        CustomerService{ domain_event_publisher: x, pool: y }
    }

    pub fn save_customer(&self, name: &String, credit_limit: i64) -> Result<i64> {
        let mut con = self.pool.get_conn().unwrap();

        let mut txn = con.start_transaction(TxOpts::default())?;

        let insert_result =
            txn.exec_drop("insert into eventuate.customers(name, credit_limit) values(?, ?)",
                          (name, credit_limit,))?;

        let id = get_last_insert_id(&mut txn)?;

        self.domain_event_publisher .publish_event(&mut txn, "Customer".to_string(), id,
                                             &CustomerCreatedEvent { name: name.clone(), credit_limit: credit_limit })?;

        txn.commit()?;

        Ok(id)
    }


    pub fn find_customer(&self, id : i64) -> Result<Option<CustomerDTO>> {
        let mut con = self.pool.get_conn().unwrap();

        let mut txn = con.start_transaction(TxOpts::default())?;

        let qwp = "SELECT name, credit_limit from eventuate.customers where id = ? "
            .with((id,));

        let customer = txn.query_map(qwp,
                |(name, credit_limit)| {
                    CustomerDTO {id, name, credit_limit }
                },
            )?;

       txn.commit()?;

       Ok(customer.clone())
    }

}

fn get_last_insert_id(txn: &mut Transaction) -> Result<i64> {
    let s = txn.prep("SELECT LAST_INSERT_ID()", ())?;
    let exec_result = s.exec_first(&s, ())?.unwrap();
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
