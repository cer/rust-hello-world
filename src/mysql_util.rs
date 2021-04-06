

use std::sync::Arc;
use mysql::Pool;

pub fn new_connection_pool() -> Arc<Pool> {
    Arc::new(Pool::new_manual(10, 50, "mysql://root:rootpassword@localhost:3306").unwrap())
}