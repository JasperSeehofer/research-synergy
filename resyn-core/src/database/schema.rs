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

async fn apply_migration_6(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS gap_finding SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS gap_type ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS paper_ids ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS shared_terms ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS justification ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS confidence ON gap_finding TYPE float;
        DEFINE FIELD IF NOT EXISTS found_at ON gap_finding TYPE string;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 6 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_7(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS crawl_queue SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS paper_id ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS seed_paper_id ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS depth_level ON crawl_queue TYPE int;
        DEFINE FIELD IF NOT EXISTS status ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS retry_count ON crawl_queue TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS created_at ON crawl_queue TYPE string;
        DEFINE FIELD IF NOT EXISTS claimed_at ON crawl_queue TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS completed_at ON crawl_queue TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_queue_paper_seed
            ON crawl_queue FIELDS paper_id, seed_paper_id UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_queue_status
            ON crawl_queue FIELDS status;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 7 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_8(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS keyword_palette SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS keyword ON keyword_palette TYPE string;
        DEFINE FIELD IF NOT EXISTS r ON keyword_palette TYPE int;
        DEFINE FIELD IF NOT EXISTS g ON keyword_palette TYPE int;
        DEFINE FIELD IF NOT EXISTS b ON keyword_palette TYPE int;
        DEFINE FIELD IF NOT EXISTS slot_index ON keyword_palette TYPE int;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON keyword_palette TYPE string;
        DEFINE FIELD IF NOT EXISTS computed_at ON keyword_palette TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_palette_keyword ON keyword_palette FIELDS keyword UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 8 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_9(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE ANALYZER IF NOT EXISTS paper_analyzer
            TOKENIZERS blank, class
            FILTERS lowercase, ascii;

        DEFINE INDEX IF NOT EXISTS idx_paper_fts_title
            ON paper FIELDS title
            FULLTEXT ANALYZER paper_analyzer BM25;

        DEFINE INDEX IF NOT EXISTS idx_paper_fts_summary
            ON paper FIELDS summary
            FULLTEXT ANALYZER paper_analyzer BM25;

        DEFINE INDEX IF NOT EXISTS idx_paper_fts_authors
            ON paper FIELDS authors
            FULLTEXT ANALYZER paper_analyzer BM25;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 9 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_10(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS paper_similarity SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON paper_similarity TYPE string;
        DEFINE FIELD IF NOT EXISTS neighbors ON paper_similarity TYPE string;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON paper_similarity TYPE string;
        DEFINE FIELD IF NOT EXISTS computed_at ON paper_similarity TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_similarity_arxiv_id ON paper_similarity FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 10 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_11(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS graph_metrics SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON graph_metrics TYPE string;
        DEFINE FIELD IF NOT EXISTS pagerank ON graph_metrics TYPE float;
        DEFINE FIELD IF NOT EXISTS betweenness ON graph_metrics TYPE float;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON graph_metrics TYPE string;
        DEFINE FIELD IF NOT EXISTS computed_at ON graph_metrics TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_metrics_arxiv_id ON graph_metrics FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 11 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_12(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS graph_communities SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON graph_communities TYPE string;
        DEFINE FIELD IF NOT EXISTS community_id ON graph_communities TYPE int;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON graph_communities TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_communities_arxiv_id ON graph_communities FIELDS arxiv_id UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_communities_community_id ON graph_communities FIELDS community_id;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 12 DDL failed: {e}")))?;
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

    if version < 6 {
        apply_migration_6(db).await?;
        record_migration(db, 6).await?;
    }

    if version < 7 {
        apply_migration_7(db).await?;
        record_migration(db, 7).await?;
    }

    if version < 8 {
        apply_migration_8(db).await?;
        record_migration(db, 8).await?;
    }

    if version < 9 {
        apply_migration_9(db).await?;
        record_migration(db, 9).await?;
    }

    if version < 10 {
        apply_migration_10(db).await?;
        record_migration(db, 10).await?;
    }

    if version < 11 {
        apply_migration_11(db).await?;
        record_migration(db, 11).await?;
    }

    if version < 12 {
        apply_migration_12(db).await?;
        record_migration(db, 12).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::client::connect_memory;

    #[tokio::test]
    async fn test_migration_10_creates_paper_similarity_table() {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");

        // Verify paper_similarity table exists by inserting and reading back a row.
        db.query(
            "CREATE paper_similarity CONTENT { \
             arxiv_id: '2301.12345', \
             neighbors: '[]', \
             corpus_fingerprint: 'test_fp', \
             computed_at: '2026-04-08T00:00:00Z' \
             }",
        )
        .await
        .expect("insert into paper_similarity failed — table likely missing");

        let mut response = db
            .query("SELECT arxiv_id FROM paper_similarity")
            .await
            .expect("select from paper_similarity failed");
        let rows: Vec<serde_json::Value> = response.take(0).expect("deserialize failed");
        assert_eq!(rows.len(), 1, "expected 1 record in paper_similarity");
        assert_eq!(
            rows[0].get("arxiv_id").and_then(|v| v.as_str()),
            Some("2301.12345")
        );
    }

    #[tokio::test]
    async fn test_migration_11_creates_graph_metrics_table() {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");

        // Verify graph_metrics table exists by inserting and reading back a row.
        db.query(
            "CREATE graph_metrics CONTENT { \
             arxiv_id: '2301.12345', \
             pagerank: 0.5, \
             betweenness: 10.0, \
             corpus_fingerprint: 'test_fp', \
             computed_at: '2026-04-09T00:00:00Z' \
             }",
        )
        .await
        .expect("insert into graph_metrics failed — table likely missing");

        let mut response = db
            .query("SELECT arxiv_id FROM graph_metrics")
            .await
            .expect("select from graph_metrics failed");
        let rows: Vec<serde_json::Value> = response.take(0).expect("deserialize failed");
        assert_eq!(rows.len(), 1, "expected 1 record in graph_metrics");
        assert_eq!(
            rows[0].get("arxiv_id").and_then(|v| v.as_str()),
            Some("2301.12345")
        );
    }

    #[tokio::test]
    async fn test_migrate_schema_idempotent() {
        let db = connect_memory().await.expect("in-memory DB failed");
        // Run twice — should not error
        migrate_schema(&db).await.expect("first migration failed");
        migrate_schema(&db).await.expect("second migration failed — not idempotent");
    }

    #[tokio::test]
    async fn test_migration_12_creates_graph_communities_table() {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");

        // Verify graph_communities table exists by inserting and reading back a row.
        db.query(
            "CREATE graph_communities CONTENT { \
             arxiv_id: '2301.12345', \
             community_id: 1, \
             corpus_fingerprint: 'test_fp' \
             }",
        )
        .await
        .expect("insert into graph_communities failed — table likely missing");

        let mut response = db
            .query("SELECT arxiv_id, community_id FROM graph_communities")
            .await
            .expect("select from graph_communities failed");
        let rows: Vec<serde_json::Value> = response.take(0).expect("deserialize failed");
        assert_eq!(rows.len(), 1, "expected 1 record in graph_communities");
        assert_eq!(
            rows[0].get("arxiv_id").and_then(|v| v.as_str()),
            Some("2301.12345")
        );
        assert_eq!(
            rows[0].get("community_id").and_then(|v| v.as_i64()),
            Some(1)
        );
    }
}
