use chrono::Utc;
use surrealdb::types::{RecordId, SurrealValue};

use crate::error::ResynError;
use crate::utils::strip_version_suffix;

use super::client::Db;

/// An internal record matching the crawl_queue table schema.
/// The `id` field is an Option because SurrealDB's `SELECT *` always includes it.
/// We include it here but don't use it for updates (using WHERE-based updates instead).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct QueueRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub paper_id: String,
    pub seed_paper_id: String,
    pub depth_level: usize,
    pub status: String,
    pub retry_count: u32,
    pub created_at: String,
    pub claimed_at: Option<String>,
    pub completed_at: Option<String>,
    pub source: Option<String>,
}

impl From<QueueRecord> for QueueEntry {
    fn from(r: QueueRecord) -> Self {
        QueueEntry {
            paper_id: r.paper_id,
            seed_paper_id: r.seed_paper_id,
            depth_level: r.depth_level,
            status: r.status,
            retry_count: r.retry_count,
            created_at: r.created_at,
            claimed_at: r.claimed_at,
            completed_at: r.completed_at,
            source: r.source,
        }
    }
}

/// Public entry type returned by the repository methods.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, surrealdb::types::SurrealValue)]
#[surreal(crate = "surrealdb::types")]
pub struct QueueEntry {
    pub paper_id: String,
    pub seed_paper_id: String,
    pub depth_level: usize,
    pub status: String,
    pub retry_count: u32,
    pub created_at: String,
    pub claimed_at: Option<String>,
    pub completed_at: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct QueueCounts {
    pub total: u64,
    pub pending: u64,
    pub fetching: u64,
    pub done: u64,
    pub failed: u64,
}

pub struct CrawlQueueRepository<'a> {
    db: &'a Db,
}

/// Build the record ID for a crawl_queue entry.
/// The ID is composed from paper_id and seed_id with special chars replaced by '_'.
fn queue_record_id(paper_id: &str, seed_id: &str) -> RecordId {
    let key = format!(
        "{}_{}",
        paper_id.replace(['/', '.'], "_"),
        seed_id.replace(['/', '.'], "_")
    );
    RecordId::new("crawl_queue", key)
}

impl<'a> CrawlQueueRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Enqueue a paper for crawling, doing nothing if (paper_id, seed_paper_id) already exists.
    ///
    /// Uses a named record ID of the form `crawl_queue:⟨{paper_id}_{seed_id}⟩` so that
    /// SurrealDB's `CREATE` is idempotent — a second CREATE on the same ID is a no-op.
    pub async fn enqueue_if_absent(
        &self,
        paper_id: &str,
        seed_id: &str,
        depth: usize,
    ) -> Result<(), ResynError> {
        let paper_id: String = strip_version_suffix(paper_id);
        let seed_id: String = seed_id.to_owned();
        let now: String = Utc::now().to_rfc3339();
        let rid = queue_record_id(&paper_id, &seed_id);

        // CREATE is idempotent on named record IDs — if the ID already exists, this is a no-op.
        self.db
            .query(
                "CREATE $rid CONTENT {
                    paper_id: $paper_id,
                    seed_paper_id: $seed_id,
                    depth_level: $depth,
                    status: 'pending',
                    retry_count: 0,
                    created_at: $now,
                    claimed_at: NONE,
                    completed_at: NONE
                }",
            )
            .bind(("rid", rid))
            .bind(("paper_id", paper_id))
            .bind(("seed_id", seed_id))
            .bind(("depth", depth))
            .bind(("now", now))
            .await
            .map_err(|e| ResynError::Database(format!("enqueue_if_absent failed: {e}")))?;

        Ok(())
    }

    /// Claim the next pending entry (shallowest depth first).
    /// Returns None if no pending entries exist.
    ///
    /// Uses a multi-statement query: LET to select, UPDATE ONLY by LET variable (direct record),
    /// RETURN the updated record. `UPDATE ONLY $entry_id` operates directly on a record ID
    /// from a LET binding.
    pub async fn claim_next_pending(&self) -> Result<Option<QueueEntry>, ResynError> {
        let mut response = self
            .db
            .query(
                "
                LET $entry_id = (SELECT id, depth_level, created_at FROM crawl_queue
                                 WHERE status = 'pending'
                                 ORDER BY depth_level ASC, created_at ASC
                                 LIMIT 1)[0].id;
                IF $entry_id != NONE THEN
                    UPDATE ONLY $entry_id SET status = 'fetching', claimed_at = <string>time::now()
                      WHERE status = 'pending'
                ELSE
                    NONE
                END;
                ",
            )
            .await
            .map_err(|e| ResynError::Database(format!("claim_next_pending failed: {e}")))?;

        // Index 0 = LET result (None), index 1 = IF/UPDATE ONLY result (the updated record or NONE)
        let entry: Option<QueueEntry> = response
            .take(1)
            .map_err(|e| ResynError::Database(format!("claim_next_pending take failed: {e}")))?;

        Ok(entry)
    }

    /// Mark a queue entry as done.
    pub async fn mark_done(&self, paper_id: &str, seed_id: &str) -> Result<(), ResynError> {
        let paper_id: String = strip_version_suffix(paper_id);
        let seed_id: String = seed_id.to_owned();
        let now: String = Utc::now().to_rfc3339();
        self.db
            .query(
                "UPDATE crawl_queue
                 SET status = 'done', completed_at = $now
                 WHERE paper_id = $paper_id AND seed_paper_id = $seed_id",
            )
            .bind(("paper_id", paper_id))
            .bind(("seed_id", seed_id))
            .bind(("now", now))
            .await
            .map_err(|e| ResynError::Database(format!("mark_done failed: {e}")))?;
        Ok(())
    }

    /// Mark a queue entry as done, recording which source resolved it.
    pub async fn mark_done_with_source(
        &self,
        paper_id: &str,
        seed_id: &str,
        source: &str,
    ) -> Result<(), ResynError> {
        let paper_id: String = strip_version_suffix(paper_id);
        let seed_id: String = seed_id.to_owned();
        let now: String = Utc::now().to_rfc3339();
        self.db
            .query(
                "UPDATE crawl_queue
                 SET status = 'done', completed_at = $now, source = $source
                 WHERE paper_id = $paper_id AND seed_paper_id = $seed_id",
            )
            .bind(("paper_id", paper_id))
            .bind(("seed_id", seed_id))
            .bind(("now", now))
            .bind(("source", source.to_owned()))
            .await
            .map_err(|e| ResynError::Database(format!("mark_done_with_source failed: {e}")))?;
        Ok(())
    }

    /// Mark a queue entry as failed (increments retry_count).
    pub async fn mark_failed(&self, paper_id: &str, seed_id: &str) -> Result<(), ResynError> {
        let paper_id: String = strip_version_suffix(paper_id);
        let seed_id: String = seed_id.to_owned();
        self.db
            .query(
                "UPDATE crawl_queue
                 SET status = 'failed', retry_count = retry_count + 1
                 WHERE paper_id = $paper_id AND seed_paper_id = $seed_id",
            )
            .bind(("paper_id", paper_id))
            .bind(("seed_id", seed_id))
            .await
            .map_err(|e| ResynError::Database(format!("mark_failed failed: {e}")))?;
        Ok(())
    }

    /// Reset all 'fetching' entries back to 'pending' (crash recovery).
    /// Returns the count of reset entries.
    pub async fn reset_stale_fetching(&self) -> Result<u64, ResynError> {
        // Count first, then update.
        let count = self.count_by_status(Some("fetching")).await?;
        if count > 0 {
            self.db
                .query(
                    "UPDATE crawl_queue
                     SET status = 'pending', claimed_at = NONE
                     WHERE status = 'fetching'",
                )
                .await
                .map_err(|e| {
                    ResynError::Database(format!("reset_stale_fetching update failed: {e}"))
                })?;
        }
        Ok(count)
    }

    /// Get counts of entries per status.
    pub async fn get_counts(&self) -> Result<QueueCounts, ResynError> {
        let total = self.count_by_status(None).await?;
        let pending = self.count_by_status(Some("pending")).await?;
        let fetching = self.count_by_status(Some("fetching")).await?;
        let done = self.count_by_status(Some("done")).await?;
        let failed = self.count_by_status(Some("failed")).await?;
        Ok(QueueCounts {
            total,
            pending,
            fetching,
            done,
            failed,
        })
    }

    async fn count_by_status(&self, status: Option<&str>) -> Result<u64, ResynError> {
        let query = if let Some(s) = status {
            format!(
                "SELECT count() AS cnt FROM crawl_queue WHERE status = '{}' GROUP ALL",
                s
            )
        } else {
            "SELECT count() AS cnt FROM crawl_queue GROUP ALL".to_string()
        };

        let mut response = self
            .db
            .query(query)
            .await
            .map_err(|e| ResynError::Database(format!("count_by_status failed: {e}")))?;

        let counts: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("count_by_status take failed: {e}")))?;

        Ok(counts
            .into_iter()
            .next()
            .and_then(|v| v.get("cnt").and_then(|c| c.as_u64()))
            .unwrap_or(0))
    }

    /// Remove all entries from the queue.
    pub async fn clear_queue(&self) -> Result<(), ResynError> {
        self.db
            .query("DELETE FROM crawl_queue")
            .await
            .map_err(|e| ResynError::Database(format!("clear_queue failed: {e}")))?;
        Ok(())
    }

    /// Mark all failed entries as pending (for retry on next crawl).
    pub async fn retry_failed(&self) -> Result<u64, ResynError> {
        let count = self.count_by_status(Some("failed")).await?;
        if count > 0 {
            self.db
                .query(
                    "UPDATE crawl_queue
                     SET status = 'pending'
                     WHERE status = 'failed'",
                )
                .await
                .map_err(|e| ResynError::Database(format!("retry_failed update failed: {e}")))?;
        }
        Ok(count)
    }

    /// Check if a paper has already been completed in the queue.
    pub async fn has_completed_paper(
        &self,
        paper_id: &str,
        seed_id: &str,
    ) -> Result<bool, ResynError> {
        let paper_id: String = strip_version_suffix(paper_id);
        let seed_id: String = seed_id.to_owned();
        let rid = queue_record_id(&paper_id, &seed_id);

        let mut response = self
            .db
            .query("SELECT status FROM $rid")
            .bind(("rid", rid))
            .await
            .map_err(|e| ResynError::Database(format!("has_completed_paper failed: {e}")))?;

        let statuses: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("has_completed_paper take failed: {e}")))?;

        Ok(statuses
            .into_iter()
            .next()
            .and_then(|v| {
                v.get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s == "done")
            })
            .unwrap_or(false))
    }

    /// Count pending entries.
    pub async fn pending_count(&self) -> Result<u64, ResynError> {
        self.count_by_status(Some("pending")).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::client::connect_memory;

    #[tokio::test]
    async fn test_queue_enqueue_dedup() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        // First enqueue should succeed
        repo.enqueue_if_absent("2301.12345", "seed-paper", 1)
            .await
            .unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.total, 1, "should have 1 entry after first enqueue");

        // Same paper_id + seed_id should NOT create a duplicate
        repo.enqueue_if_absent("2301.12345", "seed-paper", 1)
            .await
            .unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(
            counts.total, 1,
            "same paper+seed should not create duplicate"
        );

        // Different seed_id SHOULD create a second entry
        repo.enqueue_if_absent("2301.12345", "other-seed", 1)
            .await
            .unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(
            counts.total, 2,
            "different seed_id should create a second entry"
        );
    }

    #[tokio::test]
    async fn test_queue_claim() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.11111", "seed", 1)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.22222", "seed", 2)
            .await
            .unwrap();

        // Claim first entry
        let entry1 = repo.claim_next_pending().await.unwrap();
        assert!(entry1.is_some(), "should claim an entry");
        let entry1 = entry1.unwrap();
        assert_eq!(entry1.status, "fetching");

        // Claim second entry (should be different)
        let entry2 = repo.claim_next_pending().await.unwrap();
        assert!(entry2.is_some(), "should claim a second entry");
        let entry2 = entry2.unwrap();
        assert_ne!(
            entry1.paper_id, entry2.paper_id,
            "second claim should return a different paper"
        );
        assert_eq!(entry2.status, "fetching");
    }

    #[tokio::test]
    async fn test_queue_claim_empty() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        let result = repo.claim_next_pending().await.unwrap();
        assert!(result.is_none(), "should return None for empty queue");
    }

    #[tokio::test]
    async fn test_queue_claim_depth_order() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        // Enqueue shallower item second to test ordering
        repo.enqueue_if_absent("2301.33333", "seed", 3)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.11111", "seed", 1)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.22222", "seed", 2)
            .await
            .unwrap();

        // Should get depth 1 first
        let entry = repo.claim_next_pending().await.unwrap().unwrap();
        assert_eq!(entry.depth_level, 1, "should claim shallowest depth first");
    }

    #[tokio::test]
    async fn test_queue_mark_done() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.12345", "seed", 1)
            .await
            .unwrap();
        repo.mark_done("2301.12345", "seed").await.unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.done, 1);
        assert_eq!(counts.pending, 0);

        // completed_at should be set — use SELECT * to get full record
        let mut response = db
            .query("SELECT * FROM crawl_queue WHERE paper_id = '2301.12345'")
            .await
            .unwrap();
        let records: Vec<QueueRecord> = response.take(0).unwrap();
        assert!(
            records[0].completed_at.is_some(),
            "completed_at should be set"
        );
    }

    #[tokio::test]
    async fn test_queue_mark_failed() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.12345", "seed", 1)
            .await
            .unwrap();
        repo.mark_failed("2301.12345", "seed").await.unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.failed, 1);
        assert_eq!(counts.pending, 0);

        // retry_count should be incremented — use SELECT * to get full record
        let mut response = db
            .query("SELECT * FROM crawl_queue WHERE paper_id = '2301.12345'")
            .await
            .unwrap();
        let records: Vec<QueueRecord> = response.take(0).unwrap();
        assert_eq!(records[0].retry_count, 1, "retry_count should be 1");
    }

    #[tokio::test]
    async fn test_queue_reset_stale() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.11111", "seed", 1)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.22222", "seed", 2)
            .await
            .unwrap();

        // Claim both (they become 'fetching')
        repo.claim_next_pending().await.unwrap();
        repo.claim_next_pending().await.unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.fetching, 2);

        // Reset stale
        let reset = repo.reset_stale_fetching().await.unwrap();
        assert_eq!(reset, 2, "should reset 2 fetching entries");

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.pending, 2);
        assert_eq!(counts.fetching, 0);

        // claimed_at should be cleared — use SELECT * to get full records
        let mut response = db.query("SELECT * FROM crawl_queue").await.unwrap();
        let records: Vec<QueueRecord> = response.take(0).unwrap();
        for rec in &records {
            assert!(rec.claimed_at.is_none(), "claimed_at should be cleared");
        }
    }

    #[tokio::test]
    async fn test_queue_get_counts() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.11111", "seed", 1)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.22222", "seed", 2)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.33333", "seed", 3)
            .await
            .unwrap();

        // Claim one (depth 1 entry = 2301.11111)
        repo.claim_next_pending().await.unwrap();
        // Mark one done
        repo.mark_done("2301.22222", "seed").await.unwrap();
        // Mark one failed
        repo.mark_failed("2301.33333", "seed").await.unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.total, 3);
        assert_eq!(counts.fetching, 1, "one should be fetching");
        assert_eq!(counts.done, 1, "one should be done");
        assert_eq!(counts.failed, 1, "one should be failed");
        assert_eq!(counts.pending, 0, "none should be pending");
    }

    #[tokio::test]
    async fn test_queue_clear() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.11111", "seed", 1)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.22222", "seed", 2)
            .await
            .unwrap();

        repo.clear_queue().await.unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.total, 0, "queue should be empty after clear");
    }

    #[tokio::test]
    async fn test_queue_retry_failed() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.11111", "seed", 1)
            .await
            .unwrap();
        repo.enqueue_if_absent("2301.22222", "seed", 2)
            .await
            .unwrap();

        repo.mark_failed("2301.11111", "seed").await.unwrap();
        repo.mark_failed("2301.22222", "seed").await.unwrap();

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.failed, 2);

        let retried = repo.retry_failed().await.unwrap();
        assert_eq!(retried, 2, "should retry 2 failed entries");

        let counts = repo.get_counts().await.unwrap();
        assert_eq!(counts.pending, 2, "all failed entries should be pending");
        assert_eq!(counts.failed, 0);
    }

    #[tokio::test]
    async fn test_mark_done_with_source() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.12345", "seed", 1)
            .await
            .unwrap();

        // Plain mark_done leaves source as None.
        repo.enqueue_if_absent("2301.99999", "seed", 1)
            .await
            .unwrap();
        repo.mark_done("2301.99999", "seed").await.unwrap();
        let mut response = db
            .query("SELECT source FROM crawl_queue WHERE paper_id = '2301.99999'")
            .await
            .unwrap();
        let rows: Vec<serde_json::Value> = response.take(0).unwrap();
        assert!(
            rows[0].get("source").map(|v| v.is_null()).unwrap_or(true),
            "plain mark_done should leave source as None"
        );

        // mark_done_with_source persists the resolving source name.
        repo.mark_done_with_source("2301.12345", "seed", "inspirehep")
            .await
            .unwrap();
        let mut response = db
            .query("SELECT source FROM crawl_queue WHERE paper_id = '2301.12345'")
            .await
            .unwrap();
        let rows: Vec<serde_json::Value> = response.take(0).unwrap();
        assert_eq!(
            rows[0].get("source").and_then(|v| v.as_str()),
            Some("inspirehep"),
            "mark_done_with_source should set source field"
        );
    }

    #[tokio::test]
    async fn test_queue_skip_existing_paper() {
        let db = connect_memory().await.unwrap();
        let repo = CrawlQueueRepository::new(&db);

        repo.enqueue_if_absent("2301.12345", "seed", 1)
            .await
            .unwrap();

        // Not done yet
        let done = repo
            .has_completed_paper("2301.12345", "seed")
            .await
            .unwrap();
        assert!(!done, "should not be done yet");

        repo.mark_done("2301.12345", "seed").await.unwrap();

        let done = repo
            .has_completed_paper("2301.12345", "seed")
            .await
            .unwrap();
        assert!(done, "should be done after mark_done");
    }
}
