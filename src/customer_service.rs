
use mysql::*;

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

        let mut txn = con.start_transaction(true, Some(IsolationLevel::RepeatableRead), Some(false)).unwrap();

        let insert_result =
            txn.prep_exec("insert into eventuate.customers(name, credit_limit) values(:name, :credit_limit)",
                          params! {"name" =>  name, "credit_limit" => credit_limit});
        insert_result?;

        let id = get_last_insert_id(&mut txn)?;

        self.domain_event_publisher .publish_event(&mut txn, "Customer".to_string(), id,
                                             &CustomerCreatedEvent { name: name.clone(), credit_limit: credit_limit })?;

        txn.commit()?;

        Ok(id)
    }


    pub fn find_customer(&self, id : i64) -> Result<CustomerDTO> {
        let mut con = self.pool.get_conn().unwrap();

        let mut txn = con.start_transaction(true, Some(IsolationLevel::RepeatableRead), Some(false)).unwrap();

        let qr : QueryResult = txn
            .prep_exec(
                "SELECT id, name, credit_limit from eventuate.customers where id = :id ", params!{id}
            )?;

        let customers : Vec<CustomerDTO> = qr.map(|row| {
          let r = row.unwrap();
            let id : i64 = r.get(0).unwrap();
            let name : String = r.get(1).unwrap();
            let credit_limit: i64 = r.get(2).unwrap();
            CustomerDTO {id, name, credit_limit }
        }).collect();

       txn.commit()?;

       let customer : &CustomerDTO = customers.first().unwrap();

       Ok(customer.clone())
    }

}

fn get_last_insert_id(txn: &mut Transaction) -> Result<i64> {
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
