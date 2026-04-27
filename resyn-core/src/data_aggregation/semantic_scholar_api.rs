use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::datamodels::paper::{DataSource, Link, Paper, Reference};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;

use super::traits::PaperSource;

pub struct SemanticScholarSource {
    client: Client,
    base_url: String,
    last_called: Option<Instant>,
    rate_limit: Duration,
    api_key: Option<String>,
    backoff_base: Duration,
    bidirectional: bool,
    max_forward_citations: usize,
}

impl SemanticScholarSource {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://api.semanticscholar.org/graph/v1".to_string(),
            last_called: None,
            rate_limit: Duration::from_millis(1100),
            api_key: None,
            backoff_base: Duration::from_secs(2),
            bidirectional: false,
            max_forward_citations: 500,
        }
    }

    /// Reads `S2_API_KEY` from the environment and sets the rate limit accordingly.
    /// Unauthenticated: 1100 ms. Authenticated: 200 ms (dedicated 5 rps tier).
    pub fn from_env(client: Client) -> Self {
        let api_key = std::env::var("S2_API_KEY").ok();
        let rate_limit = if api_key.is_some() {
            Duration::from_millis(200)
        } else {
            Duration::from_millis(1100)
        };
        Self {
            client,
            base_url: "https://api.semanticscholar.org/graph/v1".to_string(),
            last_called: None,
            rate_limit,
            api_key,
            backoff_base: Duration::from_secs(2),
            bidirectional: false,
            max_forward_citations: 500,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_rate_limit(mut self, duration: Duration) -> Self {
        self.rate_limit = duration;
        self
    }

    pub fn with_api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    /// Override the exponential-backoff base delay. Default 2 s; set to a small
    /// value in tests so 429-retry tests complete quickly.
    pub fn with_backoff_base(mut self, duration: Duration) -> Self {
        self.backoff_base = duration;
        self
    }

    /// Enable forward-citation fetching via the S2 `/citations` endpoint.
    /// When false (default), `fetch_citing_papers_inner` is a no-op.
    pub fn with_bidirectional(mut self, val: bool) -> Self {
        self.bidirectional = val;
        self
    }

    /// Cap the number of citing papers fetched per seed (default: 500).
    /// Pagination stops once the accumulator reaches this size.
    pub fn with_max_forward_citations(mut self, n: usize) -> Self {
        self.max_forward_citations = n;
        self
    }

    async fn rate_limit_check(&mut self) {
        let now = Instant::now();
        if let Some(last_call) = self.last_called {
            let elapsed = now.duration_since(last_call);
            if elapsed < self.rate_limit {
                let remaining = self.rate_limit - elapsed;
                debug!("Semantic Scholar rate limit: sleeping for {:?}", remaining);
                sleep(remaining).await;
            }
        }
        self.last_called = Some(Instant::now());
    }

    /// GET with exponential backoff on 429 (max 3 retries: 2 s → 4 s → 8 s).
    /// Honours the `Retry-After` response header when present (capped at 60 s).
    ///
    /// Takes `&self` so it can be called from both `fetch_paper` (&self) and
    /// `fetch_references` (&mut self). Does not touch `last_called`; callers
    /// handle rate limiting separately.
    async fn get_with_backoff(&self, url: &str) -> Result<reqwest::Response, ResynError> {
        const MAX_RETRIES: u32 = 3;
        let mut delay = self.backoff_base;

        for attempt in 0..=MAX_RETRIES {
            let mut builder = self.client.get(url);
            if let Some(key) = &self.api_key {
                builder = builder.header("x-api-key", key);
            }
            let response = builder
                .send()
                .await
                .map_err(|e| ResynError::SemanticScholarApi(format!("request failed: {e}")))?;

            if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Ok(response);
            }

            if attempt == MAX_RETRIES {
                break;
            }

            let sleep_dur = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .map(|d| d.min(Duration::from_secs(60)))
                .unwrap_or(delay);

            warn!(
                attempt = attempt + 1,
                backoff_secs = sleep_dur.as_secs_f32(),
                "Semantic Scholar 429 — backing off (set S2_API_KEY for a dedicated rate limit)"
            );
            sleep(sleep_dur).await;
            delay = delay.saturating_mul(2);
        }

        Err(ResynError::SemanticScholarApi(
            "rate limited after 3 retries — set S2_API_KEY env var or lower crawl depth"
                .to_string(),
        ))
    }

    fn convert_s2_paper(s2: &S2Paper) -> Paper {
        let title = s2.title.clone().unwrap_or_default();

        let authors = s2
            .authors
            .as_ref()
            .map(|a| a.iter().filter_map(|a| a.name.clone()).collect())
            .unwrap_or_default();

        let summary = s2.abstract_text.clone().unwrap_or_default();

        let arxiv_id = s2
            .external_ids
            .as_ref()
            .and_then(|ids| ids.arxiv.as_ref())
            .map(|id| strip_version_suffix(id))
            .unwrap_or_default();

        let doi = s2.external_ids.as_ref().and_then(|ids| ids.doi.clone());

        // Prefer ISO-8601 publicationDate; fall back to year as string so that
        // the `--published-before YYYY-MM-DD` lexicographic filter still works
        // (e.g. "2003" < "2014-12-31").
        let published = s2
            .publication_date
            .clone()
            .or_else(|| s2.year.map(|y| y.to_string()))
            .unwrap_or_default();

        Paper {
            title,
            authors,
            summary,
            id: arxiv_id,
            doi,
            citation_count: s2.citation_count,
            published,
            source: DataSource::SemanticScholar,
            ..Default::default()
        }
    }

    fn convert_s2_paper_to_ref(s2: &S2Paper) -> Reference {
        let title = s2.title.clone().unwrap_or_default();
        let author = s2
            .authors
            .as_ref()
            .and_then(|a| a.first())
            .and_then(|a| a.name.clone())
            .unwrap_or_default();
        let arxiv_eprint = s2
            .external_ids
            .as_ref()
            .and_then(|ids| ids.arxiv.as_ref())
            .map(|id| strip_version_suffix(id));
        let doi = s2.external_ids.as_ref().and_then(|ids| ids.doi.clone());

        let mut links = Vec::new();
        if let Some(ref eprint) = arxiv_eprint {
            // Only add a Link for new-format IDs (e.g. "2301.12345").
            // Old-format IDs (e.g. "cond-mat/0010317") contain a slash;
            // Reference::get_arxiv_id() splits the URL by '/' and takes
            // the last segment, which would lose the category prefix.
            // For old-format IDs the arxiv_eprint fallback path is correct.
            if !eprint.contains('/') {
                links.push(Link::from_url(&format!("https://arxiv.org/abs/{eprint}")));
            }
        }

        Reference {
            author,
            title,
            links,
            doi,
            arxiv_eprint,
            ..Default::default()
        }
    }

    fn convert_s2_refs(items: &[S2RefItem]) -> Vec<Reference> {
        items
            .iter()
            .map(|item| Self::convert_s2_paper_to_ref(&item.cited_paper))
            .collect()
    }

    fn convert_s2_citations(items: &[S2CitationItem]) -> Vec<Reference> {
        items
            .iter()
            .map(|item| Self::convert_s2_paper_to_ref(&item.citing_paper))
            .collect()
    }

    /// Fetch papers citing this paper from S2 `/citations` endpoint.
    /// No-op when `bidirectional == false`.
    /// Stores result in `paper.citing_papers` (transient field added in plan 02).
    pub async fn fetch_citing_papers_inner(
        &mut self,
        paper: &mut Paper,
    ) -> Result<(), ResynError> {
        if !self.bidirectional {
            return Ok(());
        }
        self.rate_limit_check().await;

        let bare_id = strip_version_suffix(&paper.id);
        let mut offset: u32 = 0;
        let limit: u32 = 500;
        let mut all_citations: Vec<S2CitationItem> = Vec::new();

        loop {
            let url = format!(
                "{}/paper/arXiv:{}/citations?fields=externalIds,title,authors,year&limit={}&offset={}",
                self.base_url, bare_id, limit, offset
            );
            debug!(url, "Fetching citing papers from Semantic Scholar");

            let response = self.get_with_backoff(&url).await?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                break;
            }
            if !response.status().is_success() {
                return Err(ResynError::SemanticScholarApi(format!(
                    "citations HTTP {}: {}",
                    response.status(),
                    paper.id
                )));
            }

            let body = response
                .text()
                .await
                .map_err(|e| ResynError::SemanticScholarApi(format!("failed to read body: {e}")))?;

            let page = serde_json::from_str::<S2CitationsPage>(&body).map_err(|e| {
                ResynError::SemanticScholarApi(format!("failed to parse response: {e}"))
            })?;

            all_citations.extend(page.data);

            // Cap enforcement: truncate and stop paginating
            if all_citations.len() >= self.max_forward_citations {
                all_citations.truncate(self.max_forward_citations);
                break;
            }

            match page.next {
                Some(next_offset) => offset = next_offset,
                None => break,
            }
        }

        paper.citing_papers = Self::convert_s2_citations(&all_citations);
        Ok(())
    }
}

#[async_trait]
impl PaperSource for SemanticScholarSource {
    // `fetch_paper` takes `&self` (trait constraint — cannot mutate `last_called`).
    // Rate limiting for this call is provided by the external SharedRateLimiter
    // in `crawl.rs::wait_for_token`. The backoff in `get_with_backoff` is self-contained.
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError> {
        let url = format!(
            "{}/paper/arXiv:{}?fields=title,authors,year,abstract,externalIds,citationCount,publicationDate",
            self.base_url, id
        );
        debug!(url, "Fetching paper from Semantic Scholar");

        let response = self.get_with_backoff(&url).await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ResynError::PaperNotFound(id.to_string()));
        }
        if !response.status().is_success() {
            return Err(ResynError::SemanticScholarApi(format!(
                "HTTP {}: {}",
                response.status(),
                id
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ResynError::SemanticScholarApi(format!("failed to read body: {e}")))?;

        let s2_paper = serde_json::from_str::<S2Paper>(&body).map_err(|e| {
            ResynError::SemanticScholarApi(format!("failed to parse response: {e}"))
        })?;

        let mut paper = Self::convert_s2_paper(&s2_paper);
        // Guard: ensure the ID is populated even if S2 returns no externalIds.ArXiv.
        if paper.id.is_empty() {
            paper.id = strip_version_suffix(id);
        }
        Ok(paper)
    }

    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
        self.rate_limit_check().await;

        let bare_id = crate::utils::strip_version_suffix(&paper.id);
        let mut offset: u32 = 0;
        let limit: u32 = 500;
        let mut all_refs: Vec<S2RefItem> = Vec::new();

        loop {
            let url = format!(
                "{}/paper/arXiv:{}/references?fields=externalIds,title,authors,year&limit={}&offset={}",
                self.base_url, bare_id, limit, offset
            );
            debug!(url, "Fetching references from Semantic Scholar");

            let response = self.get_with_backoff(&url).await?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                // Paper may have no reference endpoint entry — not an error.
                return Ok(());
            }
            if !response.status().is_success() {
                return Err(ResynError::SemanticScholarApi(format!(
                    "references HTTP {}: {}",
                    response.status(),
                    paper.id
                )));
            }

            let body = response
                .text()
                .await
                .map_err(|e| ResynError::SemanticScholarApi(format!("failed to read body: {e}")))?;

            let page = serde_json::from_str::<S2RefsPage>(&body).map_err(|e| {
                ResynError::SemanticScholarApi(format!("failed to parse response: {e}"))
            })?;

            all_refs.extend(page.data);

            match page.next {
                Some(next_offset) => offset = next_offset,
                None => break,
            }
        }

        paper.references = Self::convert_s2_refs(&all_refs);
        Ok(())
    }

    async fn fetch_citing_papers(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
        self.fetch_citing_papers_inner(paper).await
    }

    fn source_name(&self) -> &'static str {
        "semantic_scholar"
    }
}

// --- Semantic Scholar JSON deserialization types ---

#[derive(Debug, Deserialize)]
struct S2Paper {
    title: Option<String>,
    authors: Option<Vec<S2Author>>,
    year: Option<i32>,
    #[serde(rename = "abstract")]
    abstract_text: Option<String>,
    #[serde(rename = "externalIds")]
    external_ids: Option<S2ExternalIds>,
    #[serde(rename = "citationCount")]
    citation_count: Option<u32>,
    #[serde(rename = "publicationDate")]
    publication_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S2Author {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S2ExternalIds {
    #[serde(rename = "ArXiv")]
    arxiv: Option<String>,
    #[serde(rename = "DOI")]
    doi: Option<String>,
}

#[derive(Debug, Deserialize)]
struct S2RefsPage {
    data: Vec<S2RefItem>,
    next: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct S2RefItem {
    #[serde(rename = "citedPaper")]
    cited_paper: S2Paper,
}

#[derive(Debug, Deserialize)]
struct S2CitationsPage {
    data: Vec<S2CitationItem>,
    next: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct S2CitationItem {
    #[serde(rename = "citingPaper")]
    citing_paper: S2Paper,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_s2_paper_json() -> &'static str {
        r#"{
            "title": "Epidemic spreading in scale-free networks",
            "authors": [{"name": "Pastor-Satorras, R."}, {"name": "Vespignani, A."}],
            "year": 2001,
            "abstract": "A study of epidemic spreading on scale-free networks.",
            "externalIds": {"ArXiv": "cond-mat/0010317v2", "DOI": "10.1103/PhysRevLett.86.3200"},
            "citationCount": 4200,
            "publicationDate": "2001-04-02"
        }"#
    }

    fn sample_s2_refs_json() -> &'static str {
        r#"{
            "data": [
                {
                    "citedPaper": {
                        "title": "Random graphs with arbitrary degree distributions",
                        "authors": [{"name": "Newman, M. E. J."}],
                        "year": 2001,
                        "externalIds": {"ArXiv": "cond-mat/0007235", "DOI": "10.1103/PhysRevE.64.026118"}
                    }
                },
                {
                    "citedPaper": {
                        "title": "Statistical mechanics of complex networks",
                        "authors": [{"name": "Albert, R."}, {"name": "Barabasi, A.-L."}],
                        "year": 2002,
                        "externalIds": {"DOI": "10.1103/RevModPhys.74.47"}
                    }
                },
                {
                    "citedPaper": {
                        "title": "Scale-free networks",
                        "authors": [{"name": "Barabasi, A.-L."}],
                        "year": 1999,
                        "externalIds": {"ArXiv": "cond-mat/9910332"}
                    }
                }
            ],
            "next": null
        }"#
    }

    #[test]
    fn test_convert_s2_paper() {
        let s2: S2Paper = serde_json::from_str(sample_s2_paper_json()).unwrap();
        let paper = SemanticScholarSource::convert_s2_paper(&s2);

        assert_eq!(paper.title, "Epidemic spreading in scale-free networks");
        assert_eq!(paper.authors, vec!["Pastor-Satorras, R.", "Vespignani, A."]);
        assert_eq!(paper.summary, "A study of epidemic spreading on scale-free networks.");
        // Version suffix stripped
        assert_eq!(paper.id, "cond-mat/0010317");
        assert_eq!(paper.doi, Some("10.1103/PhysRevLett.86.3200".to_string()));
        assert_eq!(paper.citation_count, Some(4200));
        assert_eq!(paper.published, "2001-04-02");
        assert_eq!(paper.source, DataSource::SemanticScholar);
    }

    #[test]
    fn test_convert_s2_refs_extracts_arxiv_ids() {
        let page: S2RefsPage = serde_json::from_str(sample_s2_refs_json()).unwrap();
        let refs = SemanticScholarSource::convert_s2_refs(&page.data);

        assert_eq!(refs.len(), 3);

        // Old-format arXiv ID: eprint set, NO link (link path would strip category prefix)
        assert_eq!(refs[0].arxiv_eprint, Some("cond-mat/0007235".to_string()));
        assert_eq!(refs[0].links.len(), 0);

        // DOI-only: no arXiv eprint, no link
        assert_eq!(refs[1].arxiv_eprint, None);
        assert_eq!(refs[1].links.len(), 0);

        // Old-format arXiv ID: same — eprint only
        assert_eq!(refs[2].arxiv_eprint, Some("cond-mat/9910332".to_string()));
        assert_eq!(refs[2].links.len(), 0);
    }

    #[tokio::test]
    async fn test_rate_limit_respected() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let refs_json = r#"{"data": [], "next": null}"#;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(refs_json))
            .expect(2)
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let rate_limit_ms = 80u64;
        let mut source = SemanticScholarSource::new(client)
            .with_base_url(mock_server.uri())
            .with_rate_limit(Duration::from_millis(rate_limit_ms));

        let mut paper = Paper::new();
        paper.id = "cond-mat/0010317".to_string();

        // First call: no wait (last_called is None).
        source.fetch_references(&mut paper).await.unwrap();

        // Second call: should sleep ≈ rate_limit_ms.
        let start = std::time::Instant::now();
        source.fetch_references(&mut paper).await.unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed >= Duration::from_millis(rate_limit_ms - 20),
            "rate limit should enforce delay, elapsed: {elapsed:?}"
        );
    }
}
