use surrealdb::types::{RecordId, SurrealValue};

use crate::datamodels::analysis::{AnalysisMetadata, PaperAnalysis};
use crate::datamodels::extraction::{ExtractionMethod, TextExtractionResult};
use crate::datamodels::gap_finding::{GapFinding, GapType};
use crate::datamodels::llm_annotation::LlmAnnotation;
use crate::datamodels::paper::{DataSource, Paper};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;

use super::client::Db;

pub struct PaperRepository<'a> {
    db: &'a Db,
}

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct CitationEdgeRow {
    from_id: String,
    to_id: String,
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
            DataSource::SemanticScholar => "SemanticScholar",
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

    /// Upsert a slice of papers. No per-paper existence check; idempotent via UPSERT.
    pub async fn upsert_papers_batch(&self, papers: &[Paper]) -> Result<usize, ResynError> {
        for paper in papers {
            self.upsert_paper(paper).await?;
        }
        Ok(papers.len())
    }

    /// Insert citation edges for (from_arxiv_id, to_arxiv_id) pairs without checking
    /// whether the target paper exists — dangling edges are acceptable for bulk ingest.
    pub async fn upsert_citations_batch(
        &self,
        pairs: &[(String, String)],
    ) -> Result<usize, ResynError> {
        for (from_id, to_id) in pairs {
            let from_rid = paper_record_id(from_id);
            let to_rid = paper_record_id(to_id);
            self.db
                .query("RELATE $from->cites->$to")
                .bind(("from", from_rid))
                .bind(("to", to_rid))
                .await
                .map_err(|e| ResynError::Database(format!("batch citation failed: {e}")))?;
        }
        Ok(pairs.len())
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
            .query(
                "SELECT title, authors, summary, arxiv_id, last_updated, published, \
                 pdf_url, comment, doi, inspire_id, citation_count, source \
                 FROM paper WHERE id IN (SELECT VALUE out FROM cites WHERE in = $rid)",
            )
            .bind(("rid", rid))
            .await
            .map_err(|e| ResynError::Database(format!("get cited papers failed: {e}")))?;
        let records: Vec<PaperRecord> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse cited papers failed: {e}")))?;
        Ok(records.iter().map(|r| r.to_paper()).collect())
    }

    pub async fn get_citing_papers(&self, arxiv_id: &str) -> Result<Vec<Paper>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let rid = paper_record_id(&id);
        let mut response = self
            .db
            .query(
                "SELECT title, authors, summary, arxiv_id, last_updated, published, \
                 pdf_url, comment, doi, inspire_id, citation_count, source \
                 FROM paper WHERE id IN (SELECT VALUE in FROM cites WHERE out = $rid)",
            )
            .bind(("rid", rid))
            .await
            .map_err(|e| ResynError::Database(format!("get citing papers failed: {e}")))?;
        let records: Vec<PaperRecord> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse citing papers failed: {e}")))?;
        Ok(records.iter().map(|r| r.to_paper()).collect())
    }

    pub async fn get_all_papers(&self) -> Result<Vec<Paper>, ResynError> {
        let records: Vec<PaperRecord> = self
            .db
            .select("paper")
            .await
            .map_err(|e| ResynError::Database(format!("get all papers failed: {e}")))?;

        Ok(records.iter().map(|r| r.to_paper()).collect())
    }

    /// Return all citation edges as (from_arxiv_id, to_arxiv_id) pairs from the `cites` table.
    pub async fn get_all_citation_edges(&self) -> Result<Vec<(String, String)>, ResynError> {
        let mut response = self
            .db
            .query("SELECT in.arxiv_id AS from_id, out.arxiv_id AS to_id FROM cites")
            .await
            .map_err(|e| ResynError::Database(format!("citation edge query failed: {e}")))?;

        let rows: Vec<CitationEdgeRow> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse citation edges failed: {e}")))?;

        Ok(rows.into_iter().map(|r| (r.from_id, r.to_id)).collect())
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

// --- ExtractionRepository ---

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct ExtractionRecord {
    arxiv_id: String,
    extraction_method: String,
    abstract_text: Option<String>,
    introduction: Option<String>,
    methods: Option<String>,
    results: Option<String>,
    conclusion: Option<String>,
    is_partial: bool,
    extracted_at: String,
}

impl From<&TextExtractionResult> for ExtractionRecord {
    fn from(r: &TextExtractionResult) -> Self {
        let method_str = match r.extraction_method {
            ExtractionMethod::AbstractOnly => "AbstractOnly",
            ExtractionMethod::Ar5ivHtml => "Ar5ivHtml",
        };
        ExtractionRecord {
            arxiv_id: strip_version_suffix(&r.arxiv_id),
            extraction_method: method_str.to_string(),
            abstract_text: r.sections.abstract_text.clone(),
            introduction: r.sections.introduction.clone(),
            methods: r.sections.methods.clone(),
            results: r.sections.results.clone(),
            conclusion: r.sections.conclusion.clone(),
            is_partial: r.is_partial,
            extracted_at: r.extracted_at.clone(),
        }
    }
}

impl ExtractionRecord {
    fn to_extraction_result(&self) -> TextExtractionResult {
        use crate::datamodels::extraction::SectionMap;
        let method = match self.extraction_method.as_str() {
            "Ar5ivHtml" => ExtractionMethod::Ar5ivHtml,
            _ => ExtractionMethod::AbstractOnly,
        };
        TextExtractionResult {
            arxiv_id: self.arxiv_id.clone(),
            extraction_method: method,
            sections: SectionMap {
                abstract_text: self.abstract_text.clone(),
                introduction: self.introduction.clone(),
                methods: self.methods.clone(),
                results: self.results.clone(),
                conclusion: self.conclusion.clone(),
            },
            is_partial: self.is_partial,
            extracted_at: self.extracted_at.clone(),
        }
    }
}

fn extraction_record_id(arxiv_id: &str) -> RecordId {
    RecordId::new("text_extraction", strip_version_suffix(arxiv_id))
}

pub struct ExtractionRepository<'a> {
    db: &'a Db,
}

impl<'a> ExtractionRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_extraction(&self, result: &TextExtractionResult) -> Result<(), ResynError> {
        let arxiv_id = strip_version_suffix(&result.arxiv_id);
        let record = ExtractionRecord::from(result);

        self.db
            .query("UPSERT type::record('text_extraction', $id) CONTENT $record")
            .bind(("id", arxiv_id))
            .bind(("record", record.into_value()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert extraction failed: {e}")))?;

        Ok(())
    }

    pub async fn get_extraction(
        &self,
        arxiv_id: &str,
    ) -> Result<Option<TextExtractionResult>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let result: Option<ExtractionRecord> = self
            .db
            .select(extraction_record_id(&id))
            .await
            .map_err(|e| ResynError::Database(format!("get extraction failed: {e}")))?;
        Ok(result.map(|r| r.to_extraction_result()))
    }

    pub async fn extraction_exists(&self, arxiv_id: &str) -> Result<bool, ResynError> {
        Ok(self.get_extraction(arxiv_id).await?.is_some())
    }

    pub async fn get_all_extractions(&self) -> Result<Vec<TextExtractionResult>, ResynError> {
        let records: Vec<ExtractionRecord> = self
            .db
            .select("text_extraction")
            .await
            .map_err(|e| ResynError::Database(format!("get all extractions failed: {e}")))?;
        Ok(records.iter().map(|r| r.to_extraction_result()).collect())
    }
}

// --- AnalysisRepository ---

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct AnalysisRecord {
    arxiv_id: String,
    tfidf_vector: serde_json::Value,
    top_terms: Vec<String>,
    top_scores: Vec<f32>,
    analyzed_at: String,
    corpus_fingerprint: String,
}

impl From<&PaperAnalysis> for AnalysisRecord {
    fn from(a: &PaperAnalysis) -> Self {
        let tfidf_value = serde_json::to_value(&a.tfidf_vector)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        AnalysisRecord {
            arxiv_id: strip_version_suffix(&a.arxiv_id),
            tfidf_vector: tfidf_value,
            top_terms: a.top_terms.clone(),
            top_scores: a.top_scores.clone(),
            analyzed_at: a.analyzed_at.clone(),
            corpus_fingerprint: a.corpus_fingerprint.clone(),
        }
    }
}

impl AnalysisRecord {
    fn to_analysis(&self) -> PaperAnalysis {
        use std::collections::HashMap;
        let tfidf_vector: HashMap<String, f32> =
            serde_json::from_value(self.tfidf_vector.clone()).unwrap_or_default();
        PaperAnalysis {
            arxiv_id: self.arxiv_id.clone(),
            tfidf_vector,
            top_terms: self.top_terms.clone(),
            top_scores: self.top_scores.clone(),
            analyzed_at: self.analyzed_at.clone(),
            corpus_fingerprint: self.corpus_fingerprint.clone(),
        }
    }
}

fn analysis_record_id(arxiv_id: &str) -> RecordId {
    RecordId::new("paper_analysis", strip_version_suffix(arxiv_id))
}

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct MetadataRecord {
    key: String,
    paper_count: i64,
    corpus_fingerprint: String,
    last_analyzed: String,
}

impl From<&AnalysisMetadata> for MetadataRecord {
    fn from(m: &AnalysisMetadata) -> Self {
        MetadataRecord {
            key: m.key.clone(),
            paper_count: m.paper_count as i64,
            corpus_fingerprint: m.corpus_fingerprint.clone(),
            last_analyzed: m.last_analyzed.clone(),
        }
    }
}

impl MetadataRecord {
    fn to_metadata(&self) -> AnalysisMetadata {
        AnalysisMetadata {
            key: self.key.clone(),
            paper_count: self.paper_count as u64,
            corpus_fingerprint: self.corpus_fingerprint.clone(),
            last_analyzed: self.last_analyzed.clone(),
        }
    }
}

fn metadata_record_id(key: &str) -> RecordId {
    RecordId::new("analysis_metadata", key)
}

pub struct AnalysisRepository<'a> {
    db: &'a Db,
}

impl<'a> AnalysisRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_analysis(&self, result: &PaperAnalysis) -> Result<(), ResynError> {
        let arxiv_id = strip_version_suffix(&result.arxiv_id);
        let record = AnalysisRecord::from(result);

        self.db
            .query("UPSERT type::record('paper_analysis', $id) CONTENT $record")
            .bind(("id", arxiv_id))
            .bind(("record", record.into_value()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert analysis failed: {e}")))?;

        Ok(())
    }

    pub async fn get_analysis(&self, arxiv_id: &str) -> Result<Option<PaperAnalysis>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let result: Option<AnalysisRecord> = self
            .db
            .select(analysis_record_id(&id))
            .await
            .map_err(|e| ResynError::Database(format!("get analysis failed: {e}")))?;
        Ok(result.map(|r| r.to_analysis()))
    }

    pub async fn analysis_exists(&self, arxiv_id: &str) -> Result<bool, ResynError> {
        Ok(self.get_analysis(arxiv_id).await?.is_some())
    }

    pub async fn get_all_analyses(&self) -> Result<Vec<PaperAnalysis>, ResynError> {
        let records: Vec<AnalysisRecord> = self
            .db
            .select("paper_analysis")
            .await
            .map_err(|e| ResynError::Database(format!("get all analyses failed: {e}")))?;
        Ok(records.iter().map(|r| r.to_analysis()).collect())
    }

    pub async fn upsert_metadata(&self, meta: &AnalysisMetadata) -> Result<(), ResynError> {
        let record = MetadataRecord::from(meta);

        self.db
            .query("UPSERT type::record('analysis_metadata', $id) CONTENT $record")
            .bind(("id", meta.key.clone()))
            .bind(("record", record.into_value()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert metadata failed: {e}")))?;

        Ok(())
    }

    pub async fn get_metadata(&self, key: &str) -> Result<Option<AnalysisMetadata>, ResynError> {
        let result: Option<MetadataRecord> = self
            .db
            .select(metadata_record_id(key))
            .await
            .map_err(|e| ResynError::Database(format!("get metadata failed: {e}")))?;
        Ok(result.map(|r| r.to_metadata()))
    }
}

// --- LlmAnnotationRepository ---

// Methods and findings are stored as JSON strings in SurrealDB SCHEMAFULL to avoid
// the nested-object field enforcement pitfall (Phase 2 lesson applied to arrays).
#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct LlmAnnotationRecord {
    arxiv_id: String,
    paper_type: String,
    methods: String,
    findings: String,
    open_problems: Vec<String>,
    provider: String,
    model_name: String,
    annotated_at: String,
}

impl From<&LlmAnnotation> for LlmAnnotationRecord {
    fn from(ann: &LlmAnnotation) -> Self {
        let methods_json = serde_json::to_string(&ann.methods).unwrap_or_else(|_| "[]".to_string());
        let findings_json =
            serde_json::to_string(&ann.findings).unwrap_or_else(|_| "[]".to_string());
        LlmAnnotationRecord {
            arxiv_id: strip_version_suffix(&ann.arxiv_id),
            paper_type: ann.paper_type.clone(),
            methods: methods_json,
            findings: findings_json,
            open_problems: ann.open_problems.clone(),
            provider: ann.provider.clone(),
            model_name: ann.model_name.clone(),
            annotated_at: ann.annotated_at.clone(),
        }
    }
}

impl LlmAnnotationRecord {
    fn to_annotation(&self) -> LlmAnnotation {
        use crate::datamodels::llm_annotation::{Finding, Method};
        let methods: Vec<Method> = serde_json::from_str(&self.methods).unwrap_or_default();
        let findings: Vec<Finding> = serde_json::from_str(&self.findings).unwrap_or_default();
        LlmAnnotation {
            arxiv_id: self.arxiv_id.clone(),
            paper_type: self.paper_type.clone(),
            methods,
            findings,
            open_problems: self.open_problems.clone(),
            provider: self.provider.clone(),
            model_name: self.model_name.clone(),
            annotated_at: self.annotated_at.clone(),
        }
    }
}

fn annotation_record_id(arxiv_id: &str) -> RecordId {
    RecordId::new("llm_annotation", strip_version_suffix(arxiv_id))
}

pub struct LlmAnnotationRepository<'a> {
    db: &'a Db,
}

impl<'a> LlmAnnotationRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_annotation(&self, ann: &LlmAnnotation) -> Result<(), ResynError> {
        let arxiv_id = strip_version_suffix(&ann.arxiv_id);
        let record = LlmAnnotationRecord::from(ann);

        self.db
            .query("UPSERT type::record('llm_annotation', $id) CONTENT $record")
            .bind(("id", arxiv_id))
            .bind(("record", record.into_value()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert annotation failed: {e}")))?;

        Ok(())
    }

    pub async fn get_annotation(
        &self,
        arxiv_id: &str,
    ) -> Result<Option<LlmAnnotation>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let result: Option<LlmAnnotationRecord> =
            self.db
                .select(annotation_record_id(&id))
                .await
                .map_err(|e| ResynError::Database(format!("get annotation failed: {e}")))?;
        Ok(result.map(|r| r.to_annotation()))
    }

    pub async fn annotation_exists(&self, arxiv_id: &str) -> Result<bool, ResynError> {
        Ok(self.get_annotation(arxiv_id).await?.is_some())
    }

    pub async fn get_all_annotations(&self) -> Result<Vec<LlmAnnotation>, ResynError> {
        let records: Vec<LlmAnnotationRecord> = self
            .db
            .select("llm_annotation")
            .await
            .map_err(|e| ResynError::Database(format!("get all annotations failed: {e}")))?;
        Ok(records.iter().map(|r| r.to_annotation()).collect())
    }
}

// --- GapFindingRepository ---

// paper_ids and shared_terms stored as JSON strings per SurrealDB SCHEMAFULL pattern
// (avoids nested-object field enforcement; consistent with LlmAnnotation lesson).
// gap_finding records use auto-generated IDs (CREATE not UPSERT) for history preservation.
#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct GapFindingRecord {
    gap_type: String,
    paper_ids: String,
    shared_terms: String,
    justification: String,
    confidence: f32,
    found_at: String,
}

impl From<&GapFinding> for GapFindingRecord {
    fn from(g: &GapFinding) -> Self {
        let paper_ids_json =
            serde_json::to_string(&g.paper_ids).unwrap_or_else(|_| "[]".to_string());
        let shared_terms_json =
            serde_json::to_string(&g.shared_terms).unwrap_or_else(|_| "[]".to_string());
        GapFindingRecord {
            gap_type: g.gap_type.as_str().to_string(),
            paper_ids: paper_ids_json,
            shared_terms: shared_terms_json,
            justification: g.justification.clone(),
            confidence: g.confidence,
            found_at: g.found_at.clone(),
        }
    }
}

impl GapFindingRecord {
    fn to_gap_finding(&self) -> GapFinding {
        let gap_type = match self.gap_type.as_str() {
            "abc_bridge" => GapType::AbcBridge,
            _ => GapType::Contradiction,
        };
        let paper_ids: Vec<String> = serde_json::from_str(&self.paper_ids).unwrap_or_default();
        let shared_terms: Vec<String> =
            serde_json::from_str(&self.shared_terms).unwrap_or_default();
        GapFinding {
            gap_type,
            paper_ids,
            shared_terms,
            justification: self.justification.clone(),
            confidence: self.confidence,
            found_at: self.found_at.clone(),
        }
    }
}

pub struct GapFindingRepository<'a> {
    db: &'a Db,
}

impl<'a> GapFindingRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// INSERT (not UPSERT) — each gap finding gets an auto-generated ID.
    /// This preserves history: multiple runs for the same paper pair create separate records.
    pub async fn insert_gap_finding(&self, finding: &GapFinding) -> Result<(), ResynError> {
        let record = GapFindingRecord::from(finding);

        self.db
            .query("CREATE gap_finding CONTENT $record")
            .bind(("record", record.into_value()))
            .await
            .map_err(|e| ResynError::Database(format!("insert gap finding failed: {e}")))?;

        Ok(())
    }

    pub async fn get_all_gap_findings(&self) -> Result<Vec<GapFinding>, ResynError> {
        let records: Vec<GapFindingRecord> = self
            .db
            .select("gap_finding")
            .await
            .map_err(|e| ResynError::Database(format!("get all gap findings failed: {e}")))?;
        Ok(records.iter().map(|r| r.to_gap_finding()).collect())
    }
}

// --- PaletteRepository ---

pub struct PaletteRepository<'a> {
    db: &'a Db,
}

impl<'a> PaletteRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_palette(
        &self,
        entries: &[(String, u8, u8, u8, u8, String)], // (keyword, r, g, b, slot_index, corpus_fingerprint)
    ) -> Result<(), ResynError> {
        // Delete existing palette entries first (full replacement per D-04)
        self.db
            .query("DELETE keyword_palette")
            .await
            .map_err(|e| ResynError::Database(format!("clear palette failed: {e}")))?;

        let now = chrono::Utc::now().to_rfc3339();
        for (keyword, r, g, b, slot_index, fingerprint) in entries {
            self.db
                .query("CREATE keyword_palette CONTENT { keyword: $keyword, r: $r, g: $g, b: $b, slot_index: $slot_index, corpus_fingerprint: $fingerprint, computed_at: $now }")
                .bind(("keyword", keyword.clone()))
                .bind(("r", *r as i64))
                .bind(("g", *g as i64))
                .bind(("b", *b as i64))
                .bind(("slot_index", *slot_index as i64))
                .bind(("fingerprint", fingerprint.clone()))
                .bind(("now", now.clone()))
                .await
                .map_err(|e| ResynError::Database(format!("upsert palette entry '{}' failed: {e}", keyword)))?;
        }
        Ok(())
    }

    pub async fn get_palette(&self) -> Result<Vec<(String, u8, u8, u8, u8)>, ResynError> {
        let mut response = self
            .db
            .query("SELECT keyword, r, g, b, slot_index FROM keyword_palette ORDER BY slot_index ASC")
            .await
            .map_err(|e| ResynError::Database(format!("get palette failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("palette deserialize failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some((
                    row.get("keyword")?.as_str()?.to_string(),
                    row.get("r")?.as_u64()? as u8,
                    row.get("g")?.as_u64()? as u8,
                    row.get("b")?.as_u64()? as u8,
                    row.get("slot_index")?.as_u64()? as u8,
                ))
            })
            .collect())
    }

    /// Get the corpus_fingerprint stored with the current palette (if any).
    /// Returns the fingerprint from the first palette entry, or None if palette is empty.
    pub async fn get_corpus_fingerprint(&self) -> Result<Option<String>, ResynError> {
        let mut response = self
            .db
            .query("SELECT corpus_fingerprint FROM keyword_palette LIMIT 1")
            .await
            .map_err(|e| ResynError::Database(format!("get corpus fingerprint failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("fingerprint deserialize failed: {e}")))?;

        Ok(rows
            .first()
            .and_then(|row| row.get("corpus_fingerprint"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }

    pub async fn clear_palette(&self) -> Result<(), ResynError> {
        self.db
            .query("DELETE keyword_palette")
            .await
            .map_err(|e| ResynError::Database(format!("clear palette failed: {e}")))?;
        Ok(())
    }
}

// --- SimilarityRepository ---

use crate::datamodels::similarity::{PaperSimilarity, SimilarNeighbor};

pub struct SimilarityRepository<'a> {
    db: &'a Db,
}

impl<'a> SimilarityRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_similarity(&self, sim: &PaperSimilarity) -> Result<(), ResynError> {
        let arxiv_id = strip_version_suffix(&sim.arxiv_id);
        // Serialize neighbors as JSON string (same pattern as LlmAnnotation methods/findings)
        // SurrealDB SCHEMAFULL TYPE string avoids nested-object field enforcement pitfalls.
        let neighbors_json =
            serde_json::to_string(&sim.neighbors).unwrap_or_else(|_| "[]".to_string());

        self.db
            .query(
                "UPSERT type::record('paper_similarity', $id) SET \
                 arxiv_id = $arxiv_id, \
                 neighbors = $neighbors, \
                 corpus_fingerprint = $corpus_fingerprint, \
                 computed_at = $computed_at",
            )
            .bind(("id", arxiv_id.clone()))
            .bind(("arxiv_id", arxiv_id))
            .bind(("neighbors", neighbors_json))
            .bind(("corpus_fingerprint", sim.corpus_fingerprint.clone()))
            .bind(("computed_at", sim.computed_at.clone()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert similarity failed: {e}")))?;

        Ok(())
    }

    pub async fn get_similarity(
        &self,
        arxiv_id: &str,
    ) -> Result<Option<PaperSimilarity>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let mut response = self
            .db
            .query("SELECT arxiv_id, neighbors, corpus_fingerprint, computed_at FROM type::record('paper_similarity', $id)")
            .bind(("id", id))
            .await
            .map_err(|e| ResynError::Database(format!("get similarity failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse similarity failed: {e}")))?;

        if let Some(row) = rows.into_iter().next() {
            let arxiv_id = row
                .get("arxiv_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let neighbors: Vec<SimilarNeighbor> = row
                .get("neighbors")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            let corpus_fingerprint = row
                .get("corpus_fingerprint")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let computed_at = row
                .get("computed_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Some(PaperSimilarity {
                arxiv_id,
                neighbors,
                corpus_fingerprint,
                computed_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_similarities(&self) -> Result<Vec<PaperSimilarity>, ResynError> {
        let mut response = self
            .db
            .query("SELECT arxiv_id, neighbors, corpus_fingerprint, computed_at FROM paper_similarity")
            .await
            .map_err(|e| ResynError::Database(format!("get all similarities failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse all similarities failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let arxiv_id = row.get("arxiv_id")?.as_str()?.to_string();
                let neighbors: Vec<SimilarNeighbor> = row
                    .get("neighbors")
                    .and_then(|v| v.as_str())
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or_default();
                let corpus_fingerprint = row
                    .get("corpus_fingerprint")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let computed_at = row
                    .get("computed_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                Some(PaperSimilarity {
                    arxiv_id,
                    neighbors,
                    corpus_fingerprint,
                    computed_at,
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod similarity_tests {
    use super::*;
    use crate::database::client::connect_memory;
    use crate::database::schema::migrate_schema;
    use crate::datamodels::similarity::{PaperSimilarity, SimilarNeighbor};

    async fn setup_db() -> crate::database::client::Db {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");
        db
    }

    fn make_similarity(arxiv_id: &str, fingerprint: &str) -> PaperSimilarity {
        PaperSimilarity {
            arxiv_id: arxiv_id.to_string(),
            neighbors: vec![
                SimilarNeighbor {
                    arxiv_id: "2301.99999".to_string(),
                    score: 0.85,
                    shared_terms: vec!["quantum".to_string(), "spin".to_string()],
                },
                SimilarNeighbor {
                    arxiv_id: "2301.88888".to_string(),
                    score: 0.72,
                    shared_terms: vec!["topology".to_string()],
                },
            ],
            corpus_fingerprint: fingerprint.to_string(),
            computed_at: "2026-04-08T10:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_similarity_upsert_get_roundtrip() {
        let db = setup_db().await;
        let repo = SimilarityRepository::new(&db);
        let sim = make_similarity("2301.12345", "fp_abc");

        repo.upsert_similarity(&sim).await.expect("upsert failed");

        let retrieved = repo
            .get_similarity("2301.12345")
            .await
            .expect("get failed")
            .expect("should exist");

        assert_eq!(retrieved.arxiv_id, sim.arxiv_id);
        assert_eq!(retrieved.corpus_fingerprint, sim.corpus_fingerprint);
        assert_eq!(retrieved.computed_at, sim.computed_at);
        assert_eq!(retrieved.neighbors.len(), sim.neighbors.len());
        assert_eq!(retrieved.neighbors[0].arxiv_id, sim.neighbors[0].arxiv_id);
        assert!(
            (retrieved.neighbors[0].score - sim.neighbors[0].score).abs() < 1e-5,
            "scores differ"
        );
        assert_eq!(retrieved.neighbors[0].shared_terms, sim.neighbors[0].shared_terms);
    }

    #[tokio::test]
    async fn test_similarity_get_nonexistent_returns_none() {
        let db = setup_db().await;
        let repo = SimilarityRepository::new(&db);

        let result = repo
            .get_similarity("9999.99999")
            .await
            .expect("get failed");
        assert!(result.is_none(), "non-existent ID should return None");
    }

    #[tokio::test]
    async fn test_similarity_get_all() {
        let db = setup_db().await;
        let repo = SimilarityRepository::new(&db);

        let sim1 = make_similarity("2301.00001", "fp1");
        let sim2 = make_similarity("2301.00002", "fp1");

        repo.upsert_similarity(&sim1).await.expect("upsert 1 failed");
        repo.upsert_similarity(&sim2).await.expect("upsert 2 failed");

        let all = repo.get_all_similarities().await.expect("get all failed");
        assert_eq!(all.len(), 2, "should have 2 records");

        let ids: Vec<&str> = all.iter().map(|s| s.arxiv_id.as_str()).collect();
        assert!(ids.contains(&"2301.00001"));
        assert!(ids.contains(&"2301.00002"));
    }

    #[tokio::test]
    async fn test_similarity_upsert_idempotent() {
        let db = setup_db().await;
        let repo = SimilarityRepository::new(&db);
        let sim = make_similarity("2301.12345", "fp_v1");

        repo.upsert_similarity(&sim).await.expect("first upsert");

        // Update with new fingerprint
        let sim_v2 = PaperSimilarity {
            corpus_fingerprint: "fp_v2".to_string(),
            ..sim.clone()
        };
        repo.upsert_similarity(&sim_v2).await.expect("second upsert");

        let retrieved = repo
            .get_similarity("2301.12345")
            .await
            .expect("get failed")
            .expect("should exist");

        assert_eq!(retrieved.corpus_fingerprint, "fp_v2", "should reflect latest upsert");

        let all = repo.get_all_similarities().await.expect("get all");
        assert_eq!(all.len(), 1, "UPSERT should not create duplicate records");
    }
}

// --- GraphMetricsRepository ---

use crate::datamodels::graph_metrics::GraphMetrics;

pub struct GraphMetricsRepository<'a> {
    db: &'a Db,
}

impl<'a> GraphMetricsRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_metrics(&self, m: &GraphMetrics) -> Result<(), ResynError> {
        let arxiv_id = strip_version_suffix(&m.arxiv_id);
        self.db
            .query(
                "UPSERT type::record('graph_metrics', $id) SET \
                 arxiv_id = $arxiv_id, \
                 pagerank = $pagerank, \
                 betweenness = $betweenness, \
                 corpus_fingerprint = $corpus_fingerprint, \
                 computed_at = $computed_at",
            )
            .bind(("id", arxiv_id.clone()))
            .bind(("arxiv_id", arxiv_id))
            .bind(("pagerank", m.pagerank as f64))
            .bind(("betweenness", m.betweenness as f64))
            .bind(("corpus_fingerprint", m.corpus_fingerprint.clone()))
            .bind(("computed_at", m.computed_at.clone()))
            .await
            .map_err(|e| ResynError::Database(format!("upsert metrics failed: {e}")))?;
        Ok(())
    }

    pub async fn get_metrics(&self, arxiv_id: &str) -> Result<Option<GraphMetrics>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let mut response = self
            .db
            .query(
                "SELECT arxiv_id, pagerank, betweenness, corpus_fingerprint, computed_at \
                 FROM type::record('graph_metrics', $id)",
            )
            .bind(("id", id))
            .await
            .map_err(|e| ResynError::Database(format!("get metrics failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse metrics failed: {e}")))?;

        if let Some(row) = rows.into_iter().next() {
            let arxiv_id = row
                .get("arxiv_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let pagerank = row
                .get("pagerank")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let betweenness = row
                .get("betweenness")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let corpus_fingerprint = row
                .get("corpus_fingerprint")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let computed_at = row
                .get("computed_at")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Ok(Some(GraphMetrics {
                arxiv_id,
                pagerank,
                betweenness,
                corpus_fingerprint,
                computed_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_metrics(&self) -> Result<Vec<GraphMetrics>, ResynError> {
        let mut response = self
            .db
            .query(
                "SELECT arxiv_id, pagerank, betweenness, corpus_fingerprint, computed_at \
                 FROM graph_metrics",
            )
            .await
            .map_err(|e| ResynError::Database(format!("get all metrics failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse all metrics failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let arxiv_id = row.get("arxiv_id")?.as_str()?.to_string();
                let pagerank = row.get("pagerank").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let betweenness =
                    row.get("betweenness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let corpus_fingerprint = row
                    .get("corpus_fingerprint")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let computed_at = row
                    .get("computed_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                Some(GraphMetrics {
                    arxiv_id,
                    pagerank,
                    betweenness,
                    corpus_fingerprint,
                    computed_at,
                })
            })
            .collect())
    }

    pub async fn get_top_by_pagerank(&self, limit: usize) -> Result<Vec<GraphMetrics>, ResynError> {
        let mut response = self
            .db
            .query(
                "SELECT arxiv_id, pagerank, betweenness, corpus_fingerprint, computed_at \
                 FROM graph_metrics ORDER BY pagerank DESC LIMIT $limit",
            )
            .bind(("limit", limit as u32))
            .await
            .map_err(|e| ResynError::Database(format!("get top metrics failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse top metrics failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let arxiv_id = row.get("arxiv_id")?.as_str()?.to_string();
                let pagerank = row.get("pagerank").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let betweenness =
                    row.get("betweenness").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let corpus_fingerprint = row
                    .get("corpus_fingerprint")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                let computed_at = row
                    .get("computed_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                Some(GraphMetrics {
                    arxiv_id,
                    pagerank,
                    betweenness,
                    corpus_fingerprint,
                    computed_at,
                })
            })
            .collect())
    }
}

// --- CommunityRepository ---

use crate::datamodels::community::CommunityAssignment;

pub struct CommunityRepository<'a> {
    db: &'a Db,
}

impl<'a> CommunityRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Upsert community assignments. Deletes existing rows for each arxiv_id in the
    /// batch first, then inserts fresh rows. Fully idempotent.
    pub async fn upsert(&self, assignments: &[CommunityAssignment]) -> Result<(), ResynError> {
        // Delete existing rows for all arxiv_ids in the batch
        for a in assignments {
            let id = strip_version_suffix(&a.arxiv_id);
            self.db
                .query(
                    "DELETE graph_communities WHERE arxiv_id = $arxiv_id",
                )
                .bind(("arxiv_id", id))
                .await
                .map_err(|e| ResynError::Database(format!("delete community row failed: {e}")))?;
        }

        // Bulk insert fresh rows
        for a in assignments {
            let id = strip_version_suffix(&a.arxiv_id);
            self.db
                .query(
                    "CREATE graph_communities CONTENT { \
                     arxiv_id: $arxiv_id, \
                     community_id: $community_id, \
                     corpus_fingerprint: $corpus_fingerprint \
                     }",
                )
                .bind(("arxiv_id", id))
                .bind(("community_id", a.community_id as i64))
                .bind(("corpus_fingerprint", a.corpus_fingerprint.clone()))
                .await
                .map_err(|e| {
                    ResynError::Database(format!("insert community row failed: {e}"))
                })?;
        }

        Ok(())
    }

    pub async fn get_by_paper(
        &self,
        arxiv_id: &str,
    ) -> Result<Option<CommunityAssignment>, ResynError> {
        let id = strip_version_suffix(arxiv_id);
        let mut response = self
            .db
            .query(
                "SELECT arxiv_id, community_id, corpus_fingerprint \
                 FROM graph_communities WHERE arxiv_id = $arxiv_id LIMIT 1",
            )
            .bind(("arxiv_id", id))
            .await
            .map_err(|e| ResynError::Database(format!("get community by paper failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse community row failed: {e}")))?;

        Ok(rows.into_iter().next().and_then(|row| {
            Some(CommunityAssignment {
                arxiv_id: row.get("arxiv_id")?.as_str()?.to_string(),
                community_id: row.get("community_id")?.as_i64()? as u32,
                corpus_fingerprint: row.get("corpus_fingerprint")?.as_str()?.to_string(),
            })
        }))
    }

    pub async fn list_all(&self) -> Result<Vec<CommunityAssignment>, ResynError> {
        let mut response = self
            .db
            .query("SELECT arxiv_id, community_id, corpus_fingerprint FROM graph_communities")
            .await
            .map_err(|e| ResynError::Database(format!("list all communities failed: {e}")))?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse all communities failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(CommunityAssignment {
                    arxiv_id: row.get("arxiv_id")?.as_str()?.to_string(),
                    community_id: row.get("community_id")?.as_i64()? as u32,
                    corpus_fingerprint: row.get("corpus_fingerprint")?.as_str()?.to_string(),
                })
            })
            .collect())
    }

    pub async fn get_by_community_id(
        &self,
        community_id: u32,
    ) -> Result<Vec<CommunityAssignment>, ResynError> {
        let mut response = self
            .db
            .query(
                "SELECT arxiv_id, community_id, corpus_fingerprint \
                 FROM graph_communities WHERE community_id = $community_id",
            )
            .bind(("community_id", community_id as i64))
            .await
            .map_err(|e| {
                ResynError::Database(format!("get by community_id failed: {e}"))
            })?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse community_id rows failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(CommunityAssignment {
                    arxiv_id: row.get("arxiv_id")?.as_str()?.to_string(),
                    community_id: row.get("community_id")?.as_i64()? as u32,
                    corpus_fingerprint: row.get("corpus_fingerprint")?.as_str()?.to_string(),
                })
            })
            .collect())
    }

    /// Returns only rows matching the given corpus fingerprint.
    /// Used by `get_community_assignments` server fn.
    pub async fn get_all_for_fingerprint(
        &self,
        fp: &str,
    ) -> Result<Vec<CommunityAssignment>, ResynError> {
        let mut response = self
            .db
            .query(
                "SELECT arxiv_id, community_id, corpus_fingerprint \
                 FROM graph_communities WHERE corpus_fingerprint = $fp",
            )
            .bind(("fp", fp.to_string()))
            .await
            .map_err(|e| {
                ResynError::Database(format!("get communities for fingerprint failed: {e}"))
            })?;

        let rows: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse fingerprint rows failed: {e}")))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(CommunityAssignment {
                    arxiv_id: row.get("arxiv_id")?.as_str()?.to_string(),
                    community_id: row.get("community_id")?.as_i64()? as u32,
                    corpus_fingerprint: row.get("corpus_fingerprint")?.as_str()?.to_string(),
                })
            })
            .collect())
    }

    /// Removes rows whose corpus_fingerprint != current_fingerprint.
    /// Returns the number of deleted rows.
    pub async fn delete_stale(&self, current_fingerprint: &str) -> Result<usize, ResynError> {
        let mut response = self
            .db
            .query(
                "SELECT COUNT() AS cnt FROM graph_communities \
                 WHERE corpus_fingerprint != $fp GROUP ALL",
            )
            .bind(("fp", current_fingerprint.to_string()))
            .await
            .map_err(|e| ResynError::Database(format!("count stale communities failed: {e}")))?;

        let counts: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("parse stale count failed: {e}")))?;

        let stale_count = counts
            .first()
            .and_then(|v| v.get("cnt"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        self.db
            .query(
                "DELETE graph_communities WHERE corpus_fingerprint != $fp",
            )
            .bind(("fp", current_fingerprint.to_string()))
            .await
            .map_err(|e| ResynError::Database(format!("delete stale communities failed: {e}")))?;

        Ok(stale_count)
    }
}

#[cfg(test)]
mod community_tests {
    use super::*;
    use crate::database::client::connect_memory;
    use crate::database::schema::migrate_schema;
    use crate::datamodels::community::CommunityAssignment;

    async fn setup_db() -> crate::database::client::Db {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");
        db
    }

    fn make_assignment(arxiv_id: &str, community_id: u32, fp: &str) -> CommunityAssignment {
        CommunityAssignment {
            arxiv_id: arxiv_id.to_string(),
            community_id,
            corpus_fingerprint: fp.to_string(),
        }
    }

    #[tokio::test]
    async fn test_community_repository_upsert_idempotent() {
        let db = setup_db().await;
        let repo = CommunityRepository::new(&db);
        let a = make_assignment("2301.12345", 1, "fp_v1");

        // Insert twice — should remain one row
        repo.upsert(&[a.clone()]).await.expect("first upsert failed");
        repo.upsert(&[a.clone()]).await.expect("second upsert failed");

        let by_paper = repo
            .get_by_paper("2301.12345")
            .await
            .expect("get_by_paper failed")
            .expect("should have a row");
        assert_eq!(by_paper.community_id, 1);
        assert_eq!(by_paper.corpus_fingerprint, "fp_v1");

        let all = repo.list_all().await.expect("list_all failed");
        assert_eq!(all.len(), 1, "upsert should not create duplicates");
    }

    #[tokio::test]
    async fn test_community_repository_delete_stale() {
        let db = setup_db().await;
        let repo = CommunityRepository::new(&db);

        // Insert with old fingerprint
        let old = make_assignment("2301.12345", 1, "old_fp");
        repo.upsert(&[old]).await.expect("upsert failed");

        // Delete stale (any row not matching "new_fp")
        let deleted = repo
            .delete_stale("new_fp")
            .await
            .expect("delete_stale failed");
        assert_eq!(deleted, 1, "should have deleted 1 stale row");

        let all = repo.list_all().await.expect("list_all failed");
        assert!(all.is_empty(), "all stale rows should be deleted");
    }

    #[tokio::test]
    async fn test_community_repository_get_by_community_id() {
        let db = setup_db().await;
        let repo = CommunityRepository::new(&db);

        let a1 = make_assignment("2301.00001", 1, "fp");
        let a2 = make_assignment("2301.00002", 1, "fp");
        let a3 = make_assignment("2301.00003", 2, "fp");

        repo.upsert(&[a1, a2, a3]).await.expect("upsert failed");

        let community1 = repo
            .get_by_community_id(1)
            .await
            .expect("get_by_community_id failed");
        assert_eq!(community1.len(), 2, "community 1 should have 2 members");

        let community2 = repo
            .get_by_community_id(2)
            .await
            .expect("get_by_community_id failed");
        assert_eq!(community2.len(), 1, "community 2 should have 1 member");
    }
}

#[cfg(test)]
mod graph_metrics_tests {
    use super::*;
    use crate::database::client::connect_memory;
    use crate::database::schema::migrate_schema;
    use crate::datamodels::graph_metrics::GraphMetrics;

    async fn setup_db() -> crate::database::client::Db {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");
        db
    }

    fn make_metrics(arxiv_id: &str, pagerank: f32, betweenness: f32) -> GraphMetrics {
        GraphMetrics {
            arxiv_id: arxiv_id.to_string(),
            pagerank,
            betweenness,
            corpus_fingerprint: "fp_test".to_string(),
            computed_at: "2026-04-09T10:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_metrics_upsert_get_roundtrip() {
        let db = setup_db().await;
        let repo = GraphMetricsRepository::new(&db);
        let m = make_metrics("2301.12345", 0.42, 15.0);

        repo.upsert_metrics(&m).await.expect("upsert failed");

        let retrieved = repo
            .get_metrics("2301.12345")
            .await
            .expect("get failed")
            .expect("should exist");

        assert_eq!(retrieved.arxiv_id, m.arxiv_id);
        assert_eq!(retrieved.corpus_fingerprint, m.corpus_fingerprint);
        assert_eq!(retrieved.computed_at, m.computed_at);
        assert!((retrieved.pagerank - m.pagerank).abs() < 1e-5, "pagerank mismatch");
        assert!((retrieved.betweenness - m.betweenness).abs() < 1e-4, "betweenness mismatch");
    }

    #[tokio::test]
    async fn test_metrics_get_nonexistent_returns_none() {
        let db = setup_db().await;
        let repo = GraphMetricsRepository::new(&db);

        let result = repo.get_metrics("9999.99999").await.expect("get failed");
        assert!(result.is_none(), "non-existent ID should return None");
    }

    #[tokio::test]
    async fn test_metrics_get_all() {
        let db = setup_db().await;
        let repo = GraphMetricsRepository::new(&db);

        repo.upsert_metrics(&make_metrics("2301.00001", 0.5, 10.0))
            .await
            .expect("upsert 1 failed");
        repo.upsert_metrics(&make_metrics("2301.00002", 0.3, 5.0))
            .await
            .expect("upsert 2 failed");

        let all = repo.get_all_metrics().await.expect("get all failed");
        assert_eq!(all.len(), 2, "should have 2 records");

        let ids: Vec<&str> = all.iter().map(|m| m.arxiv_id.as_str()).collect();
        assert!(ids.contains(&"2301.00001"));
        assert!(ids.contains(&"2301.00002"));
    }

    #[tokio::test]
    async fn test_metrics_get_top_by_pagerank() {
        let db = setup_db().await;
        let repo = GraphMetricsRepository::new(&db);

        repo.upsert_metrics(&make_metrics("2301.00001", 0.9, 10.0))
            .await
            .unwrap();
        repo.upsert_metrics(&make_metrics("2301.00002", 0.5, 5.0))
            .await
            .unwrap();
        repo.upsert_metrics(&make_metrics("2301.00003", 0.7, 20.0))
            .await
            .unwrap();
        repo.upsert_metrics(&make_metrics("2301.00004", 0.2, 1.0))
            .await
            .unwrap();

        let top3 = repo.get_top_by_pagerank(3).await.expect("get top failed");
        assert_eq!(top3.len(), 3, "should return exactly 3 records");
        // Should be ordered pagerank DESC: 0.9, 0.7, 0.5
        assert!(top3[0].pagerank >= top3[1].pagerank, "not sorted DESC");
        assert!(top3[1].pagerank >= top3[2].pagerank, "not sorted DESC");
        assert_eq!(top3[0].arxiv_id, "2301.00001", "highest pagerank first");
    }

    #[tokio::test]
    async fn test_metrics_upsert_idempotent() {
        let db = setup_db().await;
        let repo = GraphMetricsRepository::new(&db);
        let m = make_metrics("2301.12345", 0.42, 15.0);

        repo.upsert_metrics(&m).await.expect("first upsert");

        // Update with new values
        let m_v2 = GraphMetrics {
            pagerank: 0.99,
            betweenness: 99.0,
            corpus_fingerprint: "fp_v2".to_string(),
            ..m.clone()
        };
        repo.upsert_metrics(&m_v2).await.expect("second upsert");

        let retrieved = repo
            .get_metrics("2301.12345")
            .await
            .expect("get failed")
            .expect("should exist");

        assert!((retrieved.pagerank - 0.99).abs() < 1e-5, "should reflect latest upsert");
        assert_eq!(retrieved.corpus_fingerprint, "fp_v2");

        let all = repo.get_all_metrics().await.expect("get all");
        assert_eq!(all.len(), 1, "UPSERT should not create duplicate records");
    }

    #[tokio::test]
    async fn test_metrics_version_suffix_stripped() {
        let db = setup_db().await;
        let repo = GraphMetricsRepository::new(&db);

        let m = make_metrics("2301.12345v3", 0.5, 10.0);
        repo.upsert_metrics(&m).await.expect("upsert failed");

        // Get by bare ID — should find it
        let fetched = repo.get_metrics("2301.12345").await.expect("get failed");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().arxiv_id, "2301.12345");
    }
}

/// A single row returned by the full-text search query.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResultRow {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub published: String,
    pub score: f32,
}

pub struct SearchRepository<'a> {
    db: &'a Db,
}

impl<'a> SearchRepository<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    /// Full-text search across paper title, summary, and authors.
    /// Returns up to `limit` results ranked by BM25 relevance score (descending).
    /// Returns empty vec immediately if `query` is blank (no DB hit).
    pub async fn search_papers(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultRow>, ResynError> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let query_owned = query.to_string();
        let mut response = self
            .db
            .query(
                "
                SELECT
                    arxiv_id,
                    title,
                    authors,
                    published,
                    (search::score(0) * 2.0 + search::score(1) * 1.5 + search::score(2) * 1.0) AS score
                FROM paper
                WHERE title @0@ $query
                   OR summary @1@ $query
                   OR authors @2@ $query
                ORDER BY score DESC
                LIMIT $limit
                ",
            )
            .bind(("query", query_owned))
            .bind(("limit", limit))
            .await
            .map_err(|e| ResynError::Database(format!("search_papers query failed: {e}")))?;

        let raw: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ResynError::Database(format!("search_papers parse failed: {e}")))?;

        let rows = raw
            .into_iter()
            .filter_map(|row| {
                let arxiv_id = row.get("arxiv_id")?.as_str()?.to_string();
                let title = row.get("title")?.as_str()?.to_string();
                let authors = row
                    .get("authors")?
                    .as_array()?
                    .iter()
                    .filter_map(|a| a.as_str().map(|s| s.to_string()))
                    .collect();
                let published = row
                    .get("published")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let score = row
                    .get("score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                Some(SearchResultRow {
                    arxiv_id,
                    title,
                    authors,
                    published,
                    score,
                })
            })
            .collect();

        Ok(rows)
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

    // --- ExtractionRepository tests ---

    fn make_extraction(arxiv_id: &str) -> TextExtractionResult {
        use crate::datamodels::paper::Paper;
        let paper = Paper {
            id: arxiv_id.to_string(),
            summary: format!("Abstract text for {arxiv_id}"),
            ..Default::default()
        };
        TextExtractionResult::from_abstract(&paper)
    }

    #[tokio::test]
    async fn test_migrate_schema_creates_tables() {
        let db = crate::database::client::connect("mem://").await.unwrap();
        // Verify migrate_schema ran (connect already calls it via setup)
        // Check that text_extraction table was created by inserting an extraction
        let repo = ExtractionRepository::new(&db);
        let extraction = make_extraction("2301.12345");
        repo.upsert_extraction(&extraction).await.unwrap();
        // Also check schema_migrations has version 11 (migrations 1-11 applied)
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(versions[0], 11);
    }

    #[tokio::test]
    async fn test_migrate_schema_is_idempotent() {
        use crate::database::schema::migrate_schema;
        let db = crate::database::client::connect("mem://").await.unwrap();
        // Run migrate_schema again — should not error and version stays at 11
        migrate_schema(&db).await.unwrap();
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(versions[0], 11);
    }

    #[tokio::test]
    async fn test_extraction_upsert_and_get() {
        let db = connect_memory().await.unwrap();
        let repo = ExtractionRepository::new(&db);

        let extraction = make_extraction("2301.12345");
        repo.upsert_extraction(&extraction).await.unwrap();

        let fetched = repo.get_extraction("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.arxiv_id, "2301.12345");
        assert!(fetched.is_partial);
        assert_eq!(fetched.extraction_method, ExtractionMethod::AbstractOnly);
        assert_eq!(
            fetched.sections.abstract_text,
            Some("Abstract text for 2301.12345".to_string())
        );
    }

    #[tokio::test]
    async fn test_extraction_exists() {
        let db = connect_memory().await.unwrap();
        let repo = ExtractionRepository::new(&db);

        assert!(!repo.extraction_exists("2301.12345").await.unwrap());

        let extraction = make_extraction("2301.12345");
        repo.upsert_extraction(&extraction).await.unwrap();

        assert!(repo.extraction_exists("2301.12345").await.unwrap());
    }

    #[tokio::test]
    async fn test_extraction_version_suffix_dedup() {
        let db = connect_memory().await.unwrap();
        let repo = ExtractionRepository::new(&db);

        // Upsert with versioned ID
        let extraction = make_extraction("2301.12345v2");
        repo.upsert_extraction(&extraction).await.unwrap();

        // Get by bare ID — should find it
        let fetched = repo.get_extraction("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().arxiv_id, "2301.12345");
    }

    #[tokio::test]
    async fn test_get_all_extractions() {
        let db = connect_memory().await.unwrap();
        let repo = ExtractionRepository::new(&db);

        repo.upsert_extraction(&make_extraction("2301.11111"))
            .await
            .unwrap();
        repo.upsert_extraction(&make_extraction("2301.22222"))
            .await
            .unwrap();
        repo.upsert_extraction(&make_extraction("2301.33333"))
            .await
            .unwrap();

        let all = repo.get_all_extractions().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    // --- AnalysisRepository tests ---

    fn make_analysis(arxiv_id: &str) -> PaperAnalysis {
        use std::collections::HashMap;
        let mut tfidf = HashMap::new();
        tfidf.insert("quantum".to_string(), 0.85_f32);
        tfidf.insert("entanglement".to_string(), 0.72_f32);
        PaperAnalysis {
            arxiv_id: arxiv_id.to_string(),
            tfidf_vector: tfidf,
            top_terms: vec!["quantum".to_string(), "entanglement".to_string()],
            top_scores: vec![0.85_f32, 0.72_f32],
            analyzed_at: "2026-03-14T10:00:00Z".to_string(),
            corpus_fingerprint: "abc123".to_string(),
        }
    }

    #[tokio::test]
    async fn test_migrate_schema_applies_all_migrations() {
        let db = crate::database::client::connect("mem://").await.unwrap();
        // connect() already runs migrate_schema; verify version is now 10
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(
            versions[0], 11,
            "Expected schema version 11 after all migrations"
        );
    }

    #[tokio::test]
    async fn test_migrate_schema_idempotent_from_v2() {
        use crate::database::schema::migrate_schema;
        let db = crate::database::client::connect("mem://").await.unwrap();
        // Run migrate_schema again — should not error and version stays at 11
        migrate_schema(&db).await.unwrap();
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(versions[0], 11);
    }

    #[tokio::test]
    async fn test_analysis_upsert_and_get() {
        let db = connect_memory().await.unwrap();
        let repo = AnalysisRepository::new(&db);

        let analysis = make_analysis("2301.12345");
        repo.upsert_analysis(&analysis).await.unwrap();

        let fetched = repo.get_analysis("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.arxiv_id, "2301.12345");
        assert_eq!(fetched.top_terms, vec!["quantum", "entanglement"]);
        assert_eq!(fetched.top_scores.len(), 2);
        assert!((fetched.top_scores[0] - 0.85_f32).abs() < 1e-5);
        assert!((fetched.top_scores[1] - 0.72_f32).abs() < 1e-5);
        assert_eq!(fetched.tfidf_vector.len(), 2);
        let q_score = fetched.tfidf_vector.get("quantum").copied().unwrap_or(0.0);
        assert!((q_score - 0.85_f32).abs() < 1e-5);
        assert_eq!(fetched.corpus_fingerprint, "abc123");
    }

    #[tokio::test]
    async fn test_analysis_exists() {
        let db = connect_memory().await.unwrap();
        let repo = AnalysisRepository::new(&db);

        assert!(!repo.analysis_exists("2301.12345").await.unwrap());

        let analysis = make_analysis("2301.12345");
        repo.upsert_analysis(&analysis).await.unwrap();

        assert!(repo.analysis_exists("2301.12345").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_all_analyses() {
        let db = connect_memory().await.unwrap();
        let repo = AnalysisRepository::new(&db);

        repo.upsert_analysis(&make_analysis("2301.11111"))
            .await
            .unwrap();
        repo.upsert_analysis(&make_analysis("2301.22222"))
            .await
            .unwrap();
        repo.upsert_analysis(&make_analysis("2301.33333"))
            .await
            .unwrap();

        let all = repo.get_all_analyses().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_upsert_and_get_metadata() {
        let db = connect_memory().await.unwrap();
        let repo = AnalysisRepository::new(&db);

        let meta = AnalysisMetadata {
            key: "corpus_tfidf".to_string(),
            paper_count: 42,
            corpus_fingerprint: "deadbeef".to_string(),
            last_analyzed: "2026-03-14T10:00:00Z".to_string(),
        };
        repo.upsert_metadata(&meta).await.unwrap();

        let fetched = repo.get_metadata("corpus_tfidf").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.key, "corpus_tfidf");
        assert_eq!(fetched.paper_count, 42);
        assert_eq!(fetched.corpus_fingerprint, "deadbeef");
        assert_eq!(fetched.last_analyzed, "2026-03-14T10:00:00Z");
    }

    #[tokio::test]
    async fn test_analysis_version_suffix_stripped() {
        let db = connect_memory().await.unwrap();
        let repo = AnalysisRepository::new(&db);

        // Upsert with versioned arxiv_id
        let analysis = make_analysis("2301.12345v3");
        repo.upsert_analysis(&analysis).await.unwrap();

        // Get by bare ID — should find it
        let fetched = repo.get_analysis("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().arxiv_id, "2301.12345");
    }

    // --- LlmAnnotationRepository tests ---

    fn make_annotation(arxiv_id: &str) -> LlmAnnotation {
        use crate::datamodels::llm_annotation::{Finding, Method};
        LlmAnnotation {
            arxiv_id: arxiv_id.to_string(),
            paper_type: "theoretical".to_string(),
            methods: vec![Method {
                name: "variational".to_string(),
                category: "analytical".to_string(),
                ..Default::default()
            }],
            findings: vec![Finding {
                text: "Energy gap non-zero".to_string(),
                strength: "strong_evidence".to_string(),
                ..Default::default()
            }],
            open_problems: vec!["Extension to 3D".to_string()],
            provider: "noop".to_string(),
            model_name: "noop".to_string(),
            annotated_at: "2026-03-14T10:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_migrate_schema_applies_migration_5() {
        let db = crate::database::client::connect("mem://").await.unwrap();
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(
            versions[0], 11,
            "Expected schema version 11 after all migrations"
        );
    }

    #[tokio::test]
    async fn test_migrate_schema_idempotent_v5() {
        use crate::database::schema::migrate_schema;
        let db = crate::database::client::connect("mem://").await.unwrap();
        migrate_schema(&db).await.unwrap();
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(versions[0], 11);
    }

    #[tokio::test]
    async fn test_annotation_upsert_and_get() {
        let db = connect_memory().await.unwrap();
        let repo = LlmAnnotationRepository::new(&db);

        let ann = make_annotation("2301.12345");
        repo.upsert_annotation(&ann).await.unwrap();

        let fetched = repo.get_annotation("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.arxiv_id, "2301.12345");
        assert_eq!(fetched.paper_type, "theoretical");
        assert_eq!(fetched.methods.len(), 1);
        assert_eq!(fetched.methods[0].name, "variational");
        assert_eq!(fetched.findings.len(), 1);
        assert_eq!(fetched.findings[0].strength, "strong_evidence");
        assert_eq!(fetched.open_problems.len(), 1);
        assert_eq!(fetched.provider, "noop");
    }

    #[tokio::test]
    async fn test_annotation_exists() {
        let db = connect_memory().await.unwrap();
        let repo = LlmAnnotationRepository::new(&db);

        assert!(!repo.annotation_exists("2301.12345").await.unwrap());

        let ann = make_annotation("2301.12345");
        repo.upsert_annotation(&ann).await.unwrap();

        assert!(repo.annotation_exists("2301.12345").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_all_annotations() {
        let db = connect_memory().await.unwrap();
        let repo = LlmAnnotationRepository::new(&db);

        repo.upsert_annotation(&make_annotation("2301.11111"))
            .await
            .unwrap();
        repo.upsert_annotation(&make_annotation("2301.22222"))
            .await
            .unwrap();
        repo.upsert_annotation(&make_annotation("2301.33333"))
            .await
            .unwrap();

        let all = repo.get_all_annotations().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_annotation_version_suffix_dedup() {
        let db = connect_memory().await.unwrap();
        let repo = LlmAnnotationRepository::new(&db);

        // Upsert with versioned ID
        let ann = make_annotation("2301.12345v2");
        repo.upsert_annotation(&ann).await.unwrap();

        // Get by bare ID — should find it
        let fetched = repo.get_annotation("2301.12345").await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().arxiv_id, "2301.12345");
    }

    // --- GapFindingRepository tests ---

    fn make_gap_finding(gap_type: crate::datamodels::gap_finding::GapType) -> GapFinding {
        GapFinding {
            gap_type,
            paper_ids: vec!["2301.11111".to_string(), "2301.22222".to_string()],
            shared_terms: vec!["quantum".to_string(), "entanglement".to_string()],
            justification: "These papers contradict each other on quantum state collapse."
                .to_string(),
            confidence: 0.85,
            found_at: "2026-03-14T10:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_migrate_schema_creates_gap_finding_table() {
        let db = crate::database::client::connect("mem://").await.unwrap();
        // connect() runs migrate_schema — verify schema version is now 11
        let mut resp = db
            .query("SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1")
            .await
            .unwrap();
        let versions: Vec<u32> = resp.take("version").unwrap();
        assert_eq!(
            versions[0], 11,
            "Expected schema version 11 after all migrations"
        );
    }

    #[tokio::test]
    async fn test_gap_finding_insert_and_get_all() {
        let db = connect_memory().await.unwrap();
        let repo = GapFindingRepository::new(&db);

        let finding = make_gap_finding(GapType::Contradiction);
        repo.insert_gap_finding(&finding).await.unwrap();

        let all = repo.get_all_gap_findings().await.unwrap();
        assert_eq!(all.len(), 1);
        let fetched = &all[0];
        assert_eq!(fetched.gap_type, GapType::Contradiction);
        assert_eq!(fetched.paper_ids, vec!["2301.11111", "2301.22222"]);
        assert_eq!(fetched.shared_terms, vec!["quantum", "entanglement"]);
        assert_eq!(
            fetched.justification,
            "These papers contradict each other on quantum state collapse."
        );
        assert!((fetched.confidence - 0.85).abs() < 1e-5);
        assert_eq!(fetched.found_at, "2026-03-14T10:00:00Z");
    }

    #[tokio::test]
    async fn test_gap_finding_multiple_inserts_preserve_history() {
        let db = connect_memory().await.unwrap();
        let repo = GapFindingRepository::new(&db);

        // Two inserts for the same paper pair — both must be stored separately
        let finding1 = make_gap_finding(GapType::Contradiction);
        let finding2 = make_gap_finding(GapType::Contradiction);
        repo.insert_gap_finding(&finding1).await.unwrap();
        repo.insert_gap_finding(&finding2).await.unwrap();

        let all = repo.get_all_gap_findings().await.unwrap();
        assert_eq!(
            all.len(),
            2,
            "Both records should be stored (no upsert dedup)"
        );
    }

    #[tokio::test]
    async fn test_gap_finding_abc_bridge_stored_correctly() {
        let db = connect_memory().await.unwrap();
        let repo = GapFindingRepository::new(&db);

        let finding = make_gap_finding(GapType::AbcBridge);
        repo.insert_gap_finding(&finding).await.unwrap();

        let all = repo.get_all_gap_findings().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].gap_type, GapType::AbcBridge);
    }

    // --- PaletteRepository tests ---

    #[tokio::test]
    async fn test_migration_8_keyword_palette() {
        let db = connect_memory().await.unwrap();
        // Migration 8 runs automatically in connect_memory
        // Verify table exists by inserting a test record
        db.query("CREATE keyword_palette CONTENT { keyword: 'test', r: 232, g: 163, b: 75, slot_index: 0, corpus_fingerprint: 'fp1', computed_at: '2026-01-01' }")
            .await
            .unwrap();
        let mut res = db.query("SELECT * FROM keyword_palette").await.unwrap();
        let rows: Vec<serde_json::Value> = res.take(0).unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test]
    async fn test_palette_upsert_and_get() {
        let db = connect_memory().await.unwrap();
        let repo = PaletteRepository::new(&db);

        let entries = vec![
            (
                "monte_carlo".to_string(),
                0x56u8,
                0xc7u8,
                0x6bu8,
                0u8,
                "fp1".to_string(),
            ),
            (
                "bayesian".to_string(),
                0xe8u8,
                0xa3u8,
                0x4bu8,
                1u8,
                "fp1".to_string(),
            ),
        ];
        repo.upsert_palette(&entries).await.unwrap();

        let result = repo.get_palette().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "monte_carlo");
        assert_eq!(result[0].1, 0x56); // r
    }

    #[tokio::test]
    async fn test_palette_empty_returns_empty_vec() {
        let db = connect_memory().await.unwrap();
        let repo = PaletteRepository::new(&db);
        let result = repo.get_palette().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_palette_corpus_fingerprint() {
        let db = connect_memory().await.unwrap();
        let repo = PaletteRepository::new(&db);

        // Empty palette has no fingerprint
        let fp = repo.get_corpus_fingerprint().await.unwrap();
        assert!(fp.is_none());

        // After upsert, fingerprint is retrievable
        let entries = vec![(
            "kw1".to_string(),
            0x56u8,
            0xc7u8,
            0x6bu8,
            0u8,
            "paper_count:42".to_string(),
        )];
        repo.upsert_palette(&entries).await.unwrap();
        let fp = repo.get_corpus_fingerprint().await.unwrap();
        assert_eq!(fp, Some("paper_count:42".to_string()));
    }

    // --- SearchRepository tests ---

    fn make_search_paper(id: &str, title: &str, summary: &str, authors: &[&str]) -> Paper {
        Paper {
            title: title.to_string(),
            authors: authors.iter().map(|a| a.to_string()).collect(),
            summary: summary.to_string(),
            id: id.to_string(),
            published: "2024-01-01".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_search_papers_empty_query() {
        let db = connect_memory().await.unwrap();
        let repo = SearchRepository::new(&db);

        let results = repo.search_papers("", 10).await.unwrap();
        assert!(results.is_empty(), "Empty query should return empty vec");

        let results = repo.search_papers("   ", 10).await.unwrap();
        assert!(results.is_empty(), "Whitespace-only query should return empty vec");
    }

    #[tokio::test]
    async fn test_search_papers_returns_ranked_results() {
        let db = connect_memory().await.unwrap();
        let paper_repo = PaperRepository::new(&db);
        let search_repo = SearchRepository::new(&db);

        let p1 = make_search_paper(
            "2301.11111",
            "Quantum entanglement in spin systems",
            "We study quantum entanglement properties.",
            &["Alice Smith"],
        );
        let p2 = make_search_paper(
            "2301.22222",
            "Classical mechanics overview",
            "A review of classical mechanics with no quantum content.",
            &["Bob Jones"],
        );
        let p3 = make_search_paper(
            "2301.33333",
            "Quantum computing fundamentals",
            "Introduction to quantum computing paradigms.",
            &["Carol White"],
        );

        paper_repo.upsert_paper(&p1).await.unwrap();
        paper_repo.upsert_paper(&p2).await.unwrap();
        paper_repo.upsert_paper(&p3).await.unwrap();

        let results = search_repo.search_papers("quantum", 10).await.unwrap();
        assert!(!results.is_empty(), "Should return results for 'quantum'");
        // p1 and p3 mention quantum; p2 does not
        let ids: Vec<&str> = results.iter().map(|r| r.arxiv_id.as_str()).collect();
        assert!(ids.contains(&"2301.11111"), "p1 (quantum in title) should be in results");
        assert!(ids.contains(&"2301.33333"), "p3 (quantum in title) should be in results");
        // All scores should be > 0
        for r in &results {
            assert!(r.score >= 0.0, "Score should be non-negative");
        }
    }

    #[tokio::test]
    async fn test_search_papers_title_scores_higher() {
        let db = connect_memory().await.unwrap();
        let paper_repo = PaperRepository::new(&db);
        let search_repo = SearchRepository::new(&db);

        // Paper A: "quantum" only in title
        let p_title = make_search_paper(
            "2301.44444",
            "Quantum field theory introduction",
            "A study of field theory using standard approaches.",
            &["Author A"],
        );
        // Paper B: "quantum" only in summary
        let p_summary = make_search_paper(
            "2301.55555",
            "Field theory approaches",
            "We apply quantum field theory methods to analyze interactions.",
            &["Author B"],
        );

        paper_repo.upsert_paper(&p_title).await.unwrap();
        paper_repo.upsert_paper(&p_summary).await.unwrap();

        let results = search_repo.search_papers("quantum", 10).await.unwrap();
        assert_eq!(results.len(), 2, "Should find exactly 2 papers");

        let title_result = results.iter().find(|r| r.arxiv_id == "2301.44444");
        let summary_result = results.iter().find(|r| r.arxiv_id == "2301.55555");

        assert!(title_result.is_some(), "Title paper must be in results");
        assert!(summary_result.is_some(), "Summary paper must be in results");

        let title_score = title_result.unwrap().score;
        let summary_score = summary_result.unwrap().score;
        assert!(
            title_score >= summary_score,
            "Title match (score={title_score}) should score >= summary match (score={summary_score})"
        );
    }

    #[tokio::test]
    async fn test_search_papers_result_order() {
        let db = connect_memory().await.unwrap();
        let paper_repo = PaperRepository::new(&db);
        let search_repo = SearchRepository::new(&db);

        // Insert papers with varying relevance
        paper_repo.upsert_paper(&make_search_paper(
            "2301.66661",
            "Quantum quantum quantum",
            "Quantum everywhere in abstract.",
            &["Author X"],
        )).await.unwrap();
        paper_repo.upsert_paper(&make_search_paper(
            "2301.66662",
            "Quantum mechanics",
            "Brief mention of classical physics.",
            &["Author Y"],
        )).await.unwrap();
        paper_repo.upsert_paper(&make_search_paper(
            "2301.66663",
            "Classical physics",
            "No quantum content here whatsoever.",
            &["Author Z"],
        )).await.unwrap();

        let results = search_repo.search_papers("quantum", 10).await.unwrap();

        // Results should be ordered by score descending
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results should be sorted by score DESC (index {} score {} >= index {} score {})",
                i - 1,
                results[i - 1].score,
                i,
                results[i].score
            );
        }
    }

    #[tokio::test]
    async fn test_search_papers_by_author() {
        let db = connect_memory().await.unwrap();
        let paper_repo = PaperRepository::new(&db);
        let search_repo = SearchRepository::new(&db);

        let p = make_search_paper(
            "2301.77777",
            "Some Paper Title",
            "Some paper summary content.",
            &["Alice Smith", "Bob Jones"],
        );
        paper_repo.upsert_paper(&p).await.unwrap();

        // Search by first author's first name
        let results = search_repo.search_papers("Alice", 10).await.unwrap();
        assert!(
            !results.is_empty(),
            "Should find paper by author name 'Alice'"
        );
        let ids: Vec<&str> = results.iter().map(|r| r.arxiv_id.as_str()).collect();
        assert!(ids.contains(&"2301.77777"), "Should find the paper authored by Alice Smith");
    }

    #[tokio::test]
    async fn test_search_papers_no_match() {
        let db = connect_memory().await.unwrap();
        let paper_repo = PaperRepository::new(&db);
        let search_repo = SearchRepository::new(&db);

        paper_repo.upsert_paper(&make_search_paper(
            "2301.88888",
            "Regular physics paper",
            "Nothing unusual here.",
            &["Author Normal"],
        )).await.unwrap();

        let results = search_repo.search_papers("xyznonexistent123", 10).await.unwrap();
        assert!(results.is_empty(), "Should return empty vec for unmatched query");
    }
}
