

use std::sync::Arc;
use sqlx::mysql::*;
use sqlx::Pool;

pub async fn new_connection_pool() -> sqlx::Result<Arc<Pool<MySql>>> {

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:rootpassword@localhost:3306").await?;

    Ok(Arc::new(pool ))
}