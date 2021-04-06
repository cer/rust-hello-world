
use hello_world::customer_service::CustomerService;
use hello_world::event_publishing::DomainEventPublisher;
use hello_world::mysql_util::new_connection_pool;

use std::sync::Arc;
use std::time::SystemTime;


#[test]
fn customer_service_saves_and_finds_customers() {
    let pool = new_connection_pool();
    let domain_event_publisher  = Arc::new(DomainEventPublisher{});
    let customer_service : CustomerService = CustomerService::new(&domain_event_publisher, &pool);


    let name = "Fred";
    let credit_limit = 101;


    let cid = customer_service.save_customer(&name.to_string(), credit_limit).unwrap();

    let c = customer_service.find_customer(cid).unwrap().unwrap();

    assert_eq!(cid, c.id);
    assert_eq!(credit_limit, c.credit_limit);
    assert_eq!("Fred", c.name);
}

#[test]
fn customer_service_find_non_existent_customer() {
    let pool = new_connection_pool();
    let domain_event_publisher  = Arc::new(DomainEventPublisher{});
    let customer_service : CustomerService = CustomerService::new(&domain_event_publisher, &pool);

    let id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();

    let c = customer_service.find_customer(id as i64).unwrap();

    assert_eq!(None, c);
}
