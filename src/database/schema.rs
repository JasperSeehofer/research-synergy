use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::error::ResynError;

pub async fn init_schema(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS paper SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS title ON paper TYPE string;
        DEFINE FIELD IF NOT EXISTS authors ON paper TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS summary ON paper TYPE string;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON paper TYPE string;
        DEFINE FIELD IF NOT EXISTS last_updated ON paper TYPE string;
        DEFINE FIELD IF NOT EXISTS published ON paper TYPE string;
        DEFINE FIELD IF NOT EXISTS pdf_url ON paper TYPE string;
        DEFINE FIELD IF NOT EXISTS comment ON paper TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS doi ON paper TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS inspire_id ON paper TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS citation_count ON paper TYPE option<int>;
        DEFINE FIELD IF NOT EXISTS source ON paper TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_arxiv_id ON paper FIELDS arxiv_id UNIQUE;

        DEFINE TABLE IF NOT EXISTS cites SCHEMAFULL TYPE RELATION FROM paper TO paper;
        DEFINE FIELD IF NOT EXISTS label ON cites TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS ref_title ON cites TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS ref_author ON cites TYPE option<string>;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("schema init failed: {e}")))?;

    Ok(())
}
