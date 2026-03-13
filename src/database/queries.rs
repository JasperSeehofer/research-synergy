use surrealdb::types::{RecordId, SurrealValue};

use crate::datamodels::paper::{DataSource, Paper};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;

use super::client::Db;

pub struct PaperRepository<'a> {
    db: &'a Db,
}

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct PaperRecord {
    title: String,
    authors: Vec<String>,
    summary: String,
    arxiv_id: String,
    last_updated: String,
    published: String,
    pdf_url: String,
    comment: Option<String>,
    doi: Option<String>,
    inspire_id: Option<String>,
    citation_count: Option<i64>,
    source: String,
}

impl From<&Paper> for PaperRecord {
    fn from(paper: &Paper) -> Self {
        let source = match paper.source {
            DataSource::Arxiv => "Arxiv",
            DataSource::InspireHep => "InspireHep",
            DataSource::Merged => "Merged",
        };
        PaperRecord {
            title: paper.title.clone(),
            authors: paper.authors.clone(),
            summary: paper.summary.clone(),
            arxiv_id: strip_version_suffix(&paper.id),
            last_updated: paper.last_updated.clone(),
            published: paper.published.clone(),
            pdf_url: paper.pdf_url.clone(),
            comment: paper.comment.clone(),
            doi: paper.doi.clone(),
            inspire_id: paper.inspire_id.clone(),
            citation_count: paper.citation_count.map(|c| c as i64),
            source: source.to_string(),
        }
    }
}

impl PaperRecord {
    fn to_paper(&self) -> Paper {
        let source = match self.source.as_str() {
            "InspireHep" => DataSource::InspireHep,
            "Merged" => DataSource::Merged,
            _ => DataSource::Arxiv,
        };
        Paper {
            title: self.title.clone(),
            authors: self.authors.clone(),
            summary: self.summary.clone(),
            id: self.arxiv_id.clone(),
            last_updated: self.last_updated.clone(),
            published: self.published.clone(),
            pdf_url: self.pdf_url.clone(),
            comment: self.comment.clone(),
            doi: self.doi.clone(),
            inspire_id: self.inspire_id.clone(),
            citation_count: self.citation_count.map(|c| c as u32),
            source,
            references: Vec::new(),
        }
    }
}

fn paper_record_id(arxiv_id: &str) -> RecordId {
    RecordId::new("paper", strip_version_suffix(arxiv_id))
}

impl<'a> PaperRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_paper(&self, paper: &Paper) -> Result<(), ResynError> {
        let arxiv_id = strip_version_suffix(&paper.id);
        let record = PaperRecord::from(paper);

        self.db
            .query("UPSERT type::record('paper', $id) CONTENT $record")
            .bind(("id", arxiv_id))
            .bind(("record", record.into_value()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert paper failed: {e}")))?;

        Ok(())
    }

    pub async fn upsert_citations(&self, paper: &Paper) -> Result<(), ResynError> {
        let from_id = strip_version_suffix(&paper.id);

        for reference in &paper.references {
            let to_arxiv_id = if let Some(ref eprint) = reference.arxiv_eprint {
                strip_version_suffix(eprint)
            } else if let Ok(id) = reference.get_arxiv_id() {
                strip_version_suffix(&id)
            } else {
                continue;
            };

            // Only create edge if target paper exists
            if !self.paper_exists(&to_arxiv_id).await? {
                continue;
            }

            let from_rid = paper_record_id(&from_id);
            let to_rid = paper_record_id(&to_arxiv_id);

            self.db
                .query(
                    "RELATE $from->cites->$to
                     SET label = $label, ref_title = $ref_title, ref_author = $ref_author",
                )
                .bind(("from", from_rid))
                .bind(("to", to_rid))
                .bind(("label", reference.label.clone()))
                .bind((
                    "ref_title",
                    if reference.title.is_empty() {
                        None
                    } else {
                        Some(reference.title.clone())
                    },
                ))
                .bind((
                    "ref_author",
                    if reference.author.is_empty() {
                        None
                    } else {
                        Some(reference.author.clone())
                    },
                ))
                .await
                .map_err(|e| ResynError::Database(format!("upsert citation failed: {e}")))?;
        }

        Ok(())
    }

    pub async fn get_paper(&self, arxiv_id: &str) -> Result<Option<Paper>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let result: Option<PaperRecord> = self
            .db
            .select(paper_record_id(&id))
            .await
            .map_err(|e| ResynError::Database(format!("get paper failed: {e}")))?;
        Ok(result.map(|r| r.to_paper()))
    }

    pub async fn paper_exists(&self, arxiv_id: &str) -> Result<bool, ResynError> {
        Ok(self.get_paper(arxiv_id).await?.is_some())
    }

    pub async fn get_cited_papers(&self, arxiv_id: &str) -> Result<Vec<Paper>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let rid = paper_record_id(&id);
        let mut response = self
            .db
            .query("SELECT out.arxiv_id AS to_id FROM cites WHERE in = $rid")
            .bind(("rid", rid))
            .await
            .map_err(|e| ResynError::Database(format!("get cited papers failed: {e}")))?;

        let to_ids: Vec<String> = response
            .take("to_id")
            .map_err(|e| ResynError::Database(format!("parse cited papers failed: {e}")))?;

        let mut papers = Vec::new();
        for to_id in to_ids {
            if let Some(p) = self.get_paper(&to_id).await? {
                papers.push(p);
            }
        }
        Ok(papers)
    }

    pub async fn get_citing_papers(&self, arxiv_id: &str) -> Result<Vec<Paper>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let rid = paper_record_id(&id);
        let mut response = self
            .db
            .query("SELECT in.arxiv_id AS from_id FROM cites WHERE out = $rid")
            .bind(("rid", rid))
            .await
            .map_err(|e| ResynError::Database(format!("get citing papers failed: {e}")))?;

        let from_ids: Vec<String> = response
            .take("from_id")
            .map_err(|e| ResynError::Database(format!("parse citing papers failed: {e}")))?;

        let mut papers = Vec::new();
        for from_id in from_ids {
            if let Some(p) = self.get_paper(&from_id).await? {
                papers.push(p);
            }
        }
        Ok(papers)
    }

    pub async fn get_all_papers(&self) -> Result<Vec<Paper>, ResynError> {
        let records: Vec<PaperRecord> = self
            .db
            .select("paper")
            .await
            .map_err(|e| ResynError::Database(format!("get all papers failed: {e}")))?;

        Ok(records.iter().map(|r| r.to_paper()).collect())
    }

    pub async fn get_citation_graph(
        &self,
        seed_id: &str,
        max_depth: usize,
    ) -> Result<(Vec<Paper>, Vec<(String, String)>), ResynError> {
        let id = strip_version_suffix(seed_id);

        // BFS traversal using simple queries
        let mut paper_ids = std::collections::HashSet::new();
        paper_ids.insert(id.clone());
        let mut frontier = vec![id];
        let mut edges = Vec::new();

        for _depth in 0..max_depth {
            let mut next_frontier = Vec::new();
            for pid in &frontier {
                let rid = paper_record_id(pid);
                let mut response = self
                    .db
                    .query("SELECT out.arxiv_id AS to_id FROM cites WHERE in = $rid")
                    .bind(("rid", rid))
                    .await
                    .map_err(|e| ResynError::Database(format!("edge query failed: {e}")))?;

                let to_ids: Vec<String> = response
                    .take("to_id")
                    .map_err(|e| ResynError::Database(format!("parse edges failed: {e}")))?;

                for to_id in to_ids {
                    edges.push((pid.clone(), to_id.clone()));
                    if paper_ids.insert(to_id.clone()) {
                        next_frontier.push(to_id);
                    }
                }
            }
            frontier = next_frontier;
            if frontier.is_empty() {
                break;
            }
        }

        // Collect all papers
        let mut papers = Vec::new();
        for pid in &paper_ids {
            if let Some(p) = self.get_paper(pid).await? {
                papers.push(p);
            }
        }

        Ok((papers, edges))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::client::connect_memory;
    use crate::datamodels::paper::{Link, Reference};

    fn make_test_paper(id: &str, ref_ids: &[&str]) -> Paper {
        Paper {
            title: format!("Paper {id}"),
            authors: vec!["Author".to_string()],
            summary: format!("Summary of {id}"),
            id: id.to_string(),
            references: ref_ids
                .iter()
                .map(|rid| Reference {
                    links: vec![Link::from_url(&format!("https://arxiv.org/abs/{rid}"))],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_upsert_and_get_paper() {
        let db = connect_memory().await.unwrap();
        let repo = PaperRepository::new(&db);

        let paper = make_test_paper("2301.12345", &[]);
        repo.upsert_paper(&paper).await.unwrap();

        let fetched = repo.get_paper("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.title, "Paper 2301.12345");
        assert_eq!(fetched.id, "2301.12345");
        assert_eq!(fetched.authors, vec!["Author"]);
    }

    #[tokio::test]
    async fn test_upsert_is_idempotent() {
        let db = connect_memory().await.unwrap();
        let repo = PaperRepository::new(&db);

        let paper = make_test_paper("2301.12345", &[]);
        repo.upsert_paper(&paper).await.unwrap();
        repo.upsert_paper(&paper).await.unwrap();

        let all = repo.get_all_papers().await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_paper_exists() {
        let db = connect_memory().await.unwrap();
        let repo = PaperRepository::new(&db);

        assert!(!repo.paper_exists("2301.12345").await.unwrap());

        let paper = make_test_paper("2301.12345", &[]);
        repo.upsert_paper(&paper).await.unwrap();

        assert!(repo.paper_exists("2301.12345").await.unwrap());
    }

    #[tokio::test]
    async fn test_version_suffix_dedup() {
        let db = connect_memory().await.unwrap();
        let repo = PaperRepository::new(&db);

        let paper_v1 = make_test_paper("2301.12345v1", &[]);
        let paper_v2 = make_test_paper("2301.12345v2", &[]);
        repo.upsert_paper(&paper_v1).await.unwrap();
        repo.upsert_paper(&paper_v2).await.unwrap();

        let all = repo.get_all_papers().await.unwrap();
        assert_eq!(all.len(), 1);

        assert!(repo.paper_exists("2301.12345").await.unwrap());
    }

    #[tokio::test]
    async fn test_upsert_citations() {
        let db = connect_memory().await.unwrap();
        let repo = PaperRepository::new(&db);

        let paper_a = make_test_paper("2301.11111", &["2301.22222"]);
        let paper_b = make_test_paper("2301.22222", &[]);

        repo.upsert_paper(&paper_a).await.unwrap();
        repo.upsert_paper(&paper_b).await.unwrap();
        repo.upsert_citations(&paper_a).await.unwrap();

        // Verify the edge exists by checking cited papers
        let cited = repo.get_cited_papers("2301.11111").await.unwrap();
        assert_eq!(cited.len(), 1);
        assert_eq!(cited[0].id, "2301.22222");
    }

    #[tokio::test]
    async fn test_get_citation_graph() {
        let db = connect_memory().await.unwrap();
        let repo = PaperRepository::new(&db);

        // A -> B -> C chain
        let paper_a = make_test_paper("2301.11111", &["2301.22222"]);
        let paper_b = make_test_paper("2301.22222", &["2301.33333"]);
        let paper_c = make_test_paper("2301.33333", &[]);

        repo.upsert_paper(&paper_a).await.unwrap();
        repo.upsert_paper(&paper_b).await.unwrap();
        repo.upsert_paper(&paper_c).await.unwrap();
        repo.upsert_citations(&paper_a).await.unwrap();
        repo.upsert_citations(&paper_b).await.unwrap();

        let (papers, edges) = repo.get_citation_graph("2301.11111", 2).await.unwrap();

        assert_eq!(papers.len(), 3);
        assert_eq!(edges.len(), 2);
    }
}
