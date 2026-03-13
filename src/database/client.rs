use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::error::ResynError;

use super::schema::init_schema;

pub type Db = Surreal<Any>;

async fn setup(db: &Db) -> Result<(), ResynError> {
    db.use_ns("resyn")
        .use_db("resyn")
        .await
        .map_err(|e| ResynError::Database(format!("namespace/db setup failed: {e}")))?;
    init_schema(db).await?;
    Ok(())
}

pub async fn connect(endpoint: &str) -> Result<Db, ResynError> {
    let db = surrealdb::engine::any::connect(endpoint)
        .await
        .map_err(|e| ResynError::Database(format!("connection failed: {e}")))?;
    setup(&db).await?;
    Ok(db)
}

pub async fn connect_memory() -> Result<Db, ResynError> {
    connect("mem://").await
}

pub async fn connect_local(path: &str) -> Result<Db, ResynError> {
    connect(&format!("surrealkv://{path}")).await
}
