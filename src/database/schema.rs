use chrono::Utc;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::error::ResynError;

async fn get_schema_version(db: &Surreal<Any>) -> Result<u32, ResynError> {
    let mut response = db
        .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
        .await
        .map_err(|e| ResynError::Database(format!("get schema version failed: {e}")))?;

    let versions: Vec<u32> = response
        .take("version")
        .map_err(|e| ResynError::Database(format!("parse schema version failed: {e}")))?;

    Ok(versions.into_iter().next().unwrap_or(0))
}

async fn record_migration(db: &Surreal<Any>, version: u32) -> Result<(), ResynError> {
    let now = Utc::now().to_rfc3339();
    db.query("CREATE schema_migrations CONTENT { version: $version, applied_at: $now }")
        .bind(("version", version))
        .bind(("now", now))
        .await
        .map_err(|e| ResynError::Database(format!("record migration {version} failed: {e}")))?;
    Ok(())
}

async fn apply_migration_1(db: &Surreal<Any>) -> Result<(), ResynError> {
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
    .map_err(|e| ResynError::Database(format!("migration 1 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_2(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS text_extraction SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON text_extraction TYPE string;
        DEFINE FIELD IF NOT EXISTS extraction_method ON text_extraction TYPE string;
        DEFINE FIELD IF NOT EXISTS abstract_text ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS introduction ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS methods ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS results ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS conclusion ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS is_partial ON text_extraction TYPE bool;
        DEFINE FIELD IF NOT EXISTS extracted_at ON text_extraction TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_extraction_arxiv_id ON text_extraction FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 2 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_3(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS paper_analysis SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON paper_analysis TYPE string;
        DEFINE FIELD IF NOT EXISTS tfidf_vector ON paper_analysis TYPE object FLEXIBLE;
        DEFINE FIELD IF NOT EXISTS top_terms ON paper_analysis TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS top_scores ON paper_analysis TYPE array<float>;
        DEFINE FIELD IF NOT EXISTS analyzed_at ON paper_analysis TYPE string;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON paper_analysis TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_analysis_arxiv_id ON paper_analysis FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 3 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_4(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS analysis_metadata SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS key ON analysis_metadata TYPE string;
        DEFINE FIELD IF NOT EXISTS paper_count ON analysis_metadata TYPE int;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON analysis_metadata TYPE string;
        DEFINE FIELD IF NOT EXISTS last_analyzed ON analysis_metadata TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_metadata_key ON analysis_metadata FIELDS key UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 4 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_5(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS llm_annotation SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON llm_annotation TYPE string;
        DEFINE FIELD IF NOT EXISTS paper_type ON llm_annotation TYPE string;
        DEFINE FIELD IF NOT EXISTS methods ON llm_annotation TYPE string;
        DEFINE FIELD IF NOT EXISTS findings ON llm_annotation TYPE string;
        DEFINE FIELD IF NOT EXISTS open_problems ON llm_annotation TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS provider ON llm_annotation TYPE string;
        DEFINE FIELD IF NOT EXISTS model_name ON llm_annotation TYPE string;
        DEFINE FIELD IF NOT EXISTS annotated_at ON llm_annotation TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_llm_annotation_arxiv_id ON llm_annotation FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 5 DDL failed: {e}")))?;
    Ok(())
}

pub async fn migrate_schema(db: &Surreal<Any>) -> Result<(), ResynError> {
    // Ensure migrations table exists first
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS schema_migrations SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS version ON schema_migrations TYPE int;
        DEFINE FIELD IF NOT EXISTS applied_at ON schema_migrations TYPE string;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("create schema_migrations table failed: {e}")))?;

    let version = get_schema_version(db).await?;

    if version < 1 {
        apply_migration_1(db).await?;
        record_migration(db, 1).await?;
    }

    if version < 2 {
        apply_migration_2(db).await?;
        record_migration(db, 2).await?;
    }

    if version < 3 {
        apply_migration_3(db).await?;
        record_migration(db, 3).await?;
    }

    if version < 4 {
        apply_migration_4(db).await?;
        record_migration(db, 4).await?;
    }

    if version < 5 {
        apply_migration_5(db).await?;
        record_migration(db, 5).await?;
    }

    Ok(())
}
