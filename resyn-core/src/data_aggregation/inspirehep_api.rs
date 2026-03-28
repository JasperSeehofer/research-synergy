use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Deserializer};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::debug;

use crate::datamodels::paper::{DataSource, Link, Paper, Reference};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;

use super::traits::PaperSource;

pub struct InspireHepClient {
    client: Client,
    base_url: String,
    last_called: Option<Instant>,
    rate_limit: Duration,
}

impl InspireHepClient {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            base_url: "https://inspirehep.net/api".to_string(),
            last_called: None,
            rate_limit: Duration::from_millis(350),
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

    async fn rate_limit_check(&mut self) {
        let now = Instant::now();
        if let Some(last_call) = self.last_called {
            let elapsed = now.duration_since(last_call);
            if elapsed < self.rate_limit {
                let remaining = self.rate_limit - elapsed;
                debug!("InspireHEP rate limit: sleeping for {:?}", remaining);
                sleep(remaining).await;
            }
        }
        self.last_called = Some(Instant::now());
    }

    async fn fetch_literature(&mut self, arxiv_id: &str) -> Result<InspireResponse, ResynError> {
        self.rate_limit_check().await;
        let url = format!(
            "{}/literature?q=arxiv:{}&fields=references,titles,authors,abstracts,arxiv_eprints,dois,citation_count,earliest_date",
            self.base_url, arxiv_id
        );
        debug!(url, "Fetching from InspireHEP");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ResynError::InspireHepApi(format!("request failed: {e}")))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ResynError::PaperNotFound(arxiv_id.to_string()));
        }

        if !response.status().is_success() {
            return Err(ResynError::InspireHepApi(format!(
                "HTTP {}: {}",
                response.status(),
                arxiv_id
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ResynError::InspireHepApi(format!("failed to read body: {e}")))?;

        serde_json::from_str::<InspireResponse>(&body)
            .map_err(|e| ResynError::InspireHepApi(format!("failed to parse response: {e}")))
    }

    fn convert_hit_to_paper(hit: &InspireHit) -> Paper {
        let metadata = &hit.metadata;

        let title = metadata
            .titles
            .as_ref()
            .and_then(|t| t.first())
            .map(|t| t.title.clone())
            .unwrap_or_default();

        let authors = metadata
            .authors
            .as_ref()
            .map(|authors| authors.iter().map(|a| a.full_name.clone()).collect())
            .unwrap_or_default();

        let summary = metadata
            .abstracts
            .as_ref()
            .and_then(|a| a.first())
            .map(|a| a.value.clone())
            .unwrap_or_default();

        let arxiv_id = metadata
            .arxiv_eprints
            .as_ref()
            .and_then(|e| e.first())
            .map(|e| strip_version_suffix(&e.value))
            .unwrap_or_default();

        let doi = metadata
            .dois
            .as_ref()
            .and_then(|d| d.first())
            .map(|d| d.value.clone());

        let inspire_id = hit.id.as_ref().map(|id| id.to_string());

        let citation_count = metadata.citation_count;

        let published = metadata
            .earliest_date
            .as_deref()
            .unwrap_or_default()
            .to_string();
        debug!(
            "Set published date {} from InspireHEP earliest_date for paper {}",
            published, arxiv_id
        );

        Paper {
            title,
            authors,
            summary,
            id: arxiv_id,
            doi,
            inspire_id,
            citation_count,
            published,
            source: DataSource::InspireHep,
            ..Default::default()
        }
    }

    fn convert_references(metadata: &InspireMetadata) -> Vec<Reference> {
        let Some(refs) = &metadata.references else {
            return Vec::new();
        };

        refs.iter()
            .map(|r| {
                let ref_detail = &r.reference;
                let title = ref_detail
                    .as_ref()
                    .and_then(|d| d.title.as_ref())
                    .map(|t| t.title.clone())
                    .unwrap_or_default();

                let author = ref_detail
                    .as_ref()
                    .and_then(|d| d.authors.as_ref())
                    .and_then(|a| a.first())
                    .map(|a| a.full_name.clone())
                    .unwrap_or_default();

                let arxiv_eprint = ref_detail.as_ref().and_then(|d| d.arxiv_eprint.clone());

                let doi = ref_detail
                    .as_ref()
                    .and_then(|d| d.dois.as_ref())
                    .and_then(|d| d.first())
                    .cloned();

                let label = r.label.clone();

                let inspire_record_id = r
                    .record
                    .as_ref()
                    .and_then(|rec| rec.ref_url.as_ref())
                    .and_then(|url| url.rsplit('/').next())
                    .map(|s| s.to_string());

                let mut links = Vec::new();
                if let Some(ref eprint) = arxiv_eprint {
                    links.push(Link::from_url(&format!("https://arxiv.org/abs/{}", eprint)));
                }

                Reference {
                    author,
                    title,
                    links,
                    doi,
                    arxiv_eprint,
                    inspire_record_id,
                    label,
                }
            })
            .collect()
    }
}

#[async_trait]
impl PaperSource for InspireHepClient {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError> {
        // We need mut for rate limiting, but the trait requires &self for fetch_paper.
        // Work around by creating a one-off request without rate limiting for this method.
        let url = format!(
            "{}/literature?q=arxiv:{}&fields=titles,authors,abstracts,arxiv_eprints,dois,citation_count,earliest_date",
            self.base_url, id
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ResynError::InspireHepApi(format!("request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(ResynError::PaperNotFound(id.to_string()));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ResynError::InspireHepApi(format!("failed to read body: {e}")))?;

        let resp: InspireResponse = serde_json::from_str(&body)
            .map_err(|e| ResynError::InspireHepApi(format!("failed to parse: {e}")))?;

        resp.hits
            .hits
            .first()
            .map(Self::convert_hit_to_paper)
            .ok_or_else(|| ResynError::PaperNotFound(id.to_string()))
    }

    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
        let resp = self.fetch_literature(&paper.id).await?;

        if let Some(hit) = resp.hits.hits.first() {
            paper.references = Self::convert_references(&hit.metadata);
            if paper.citation_count.is_none() {
                paper.citation_count = hit.metadata.citation_count;
            }
        }

        Ok(())
    }

    fn source_name(&self) -> &'static str {
        "inspirehep"
    }
}

// --- InspireHEP JSON deserialization types ---

#[derive(Debug, Deserialize)]
pub(crate) struct InspireResponse {
    pub hits: InspireHits,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireHits {
    pub hits: Vec<InspireHit>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireHit {
    #[serde(default, deserialize_with = "deserialize_id_flexible")]
    pub id: Option<u64>,
    pub metadata: InspireMetadata,
}

fn deserialize_id_flexible<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IdValue {
        Num(u64),
        Str(String),
    }

    Option::<IdValue>::deserialize(deserializer).and_then(|opt| match opt {
        None => Ok(None),
        Some(IdValue::Num(n)) => Ok(Some(n)),
        Some(IdValue::Str(s)) => s
            .parse::<u64>()
            .map(Some)
            .map_err(|_| de::Error::custom(format!("invalid id string: {s}"))),
    })
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireMetadata {
    pub titles: Option<Vec<InspireTitle>>,
    pub authors: Option<Vec<InspireAuthor>>,
    pub abstracts: Option<Vec<InspireAbstract>>,
    pub arxiv_eprints: Option<Vec<InspireArxivEprint>>,
    pub dois: Option<Vec<InspireDoi>>,
    pub citation_count: Option<u32>,
    pub references: Option<Vec<InspireReferenceEntry>>,
    pub earliest_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireTitle {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireAuthor {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireAbstract {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireArxivEprint {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireDoi {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireReferenceEntry {
    pub reference: Option<InspireRefDetail>,
    pub record: Option<InspireRecord>,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireRefDetail {
    pub title: Option<InspireTitle>,
    pub authors: Option<Vec<InspireAuthor>>,
    pub arxiv_eprint: Option<String>,
    pub dois: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InspireRecord {
    #[serde(rename = "$ref")]
    pub ref_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_inspire_response_json() -> &'static str {
        r#"{
            "hits": {
                "hits": [{
                    "id": 1234567,
                    "metadata": {
                        "titles": [{"title": "Test Paper Title"}],
                        "authors": [
                            {"full_name": "Doe, John"},
                            {"full_name": "Smith, Jane"}
                        ],
                        "abstracts": [{"value": "This is the abstract."}],
                        "arxiv_eprints": [{"value": "2301.12345"}],
                        "dois": [{"value": "10.1234/test.2023"}],
                        "citation_count": 42,
                        "earliest_date": "2023-01-15",
                        "references": [
                            {
                                "reference": {
                                    "title": {"title": "Referenced Paper"},
                                    "authors": [{"full_name": "Ref, Author"}],
                                    "arxiv_eprint": "2301.11111",
                                    "dois": ["10.1234/ref.2023"]
                                },
                                "record": {"$ref": "https://inspirehep.net/api/literature/9999999"},
                                "label": "1"
                            },
                            {
                                "reference": {
                                    "title": {"title": "Another Paper"},
                                    "authors": [{"full_name": "Other, Author"}]
                                },
                                "record": {"$ref": "https://inspirehep.net/api/literature/8888888"},
                                "label": "2"
                            }
                        ]
                    }
                }]
            }
        }"#
    }

    #[test]
    fn test_deserialize_inspire_response() {
        let json = sample_inspire_response_json();
        let resp: InspireResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.hits.hits.len(), 1);

        let hit = &resp.hits.hits[0];
        assert_eq!(hit.id, Some(1234567));

        let meta = &hit.metadata;
        assert_eq!(meta.titles.as_ref().unwrap()[0].title, "Test Paper Title");
        assert_eq!(meta.authors.as_ref().unwrap().len(), 2);
        assert_eq!(meta.citation_count, Some(42));
    }

    #[test]
    fn test_convert_hit_to_paper() {
        let json = sample_inspire_response_json();
        let resp: InspireResponse = serde_json::from_str(json).unwrap();
        let paper = InspireHepClient::convert_hit_to_paper(&resp.hits.hits[0]);

        assert_eq!(paper.title, "Test Paper Title");
        assert_eq!(paper.authors, vec!["Doe, John", "Smith, Jane"]);
        assert_eq!(paper.summary, "This is the abstract.");
        assert_eq!(paper.id, "2301.12345");
        assert_eq!(paper.doi, Some("10.1234/test.2023".to_string()));
        assert_eq!(paper.inspire_id, Some("1234567".to_string()));
        assert_eq!(paper.citation_count, Some(42));
        assert_eq!(paper.source, DataSource::InspireHep);
        assert_eq!(paper.published, "2023-01-15");
    }

    #[test]
    fn test_convert_references() {
        let json = sample_inspire_response_json();
        let resp: InspireResponse = serde_json::from_str(json).unwrap();
        let refs = InspireHepClient::convert_references(&resp.hits.hits[0].metadata);

        assert_eq!(refs.len(), 2);

        assert_eq!(refs[0].title, "Referenced Paper");
        assert_eq!(refs[0].author, "Ref, Author");
        assert_eq!(refs[0].arxiv_eprint, Some("2301.11111".to_string()));
        assert_eq!(refs[0].doi, Some("10.1234/ref.2023".to_string()));
        assert_eq!(refs[0].label, Some("1".to_string()));
        assert_eq!(refs[0].inspire_record_id, Some("9999999".to_string()));
        assert_eq!(refs[0].links.len(), 1);
        assert!(refs[0].links[0].url.contains("2301.11111"));

        // Second ref has no arxiv_eprint, so no links
        assert_eq!(refs[1].title, "Another Paper");
        assert_eq!(refs[1].arxiv_eprint, None);
        assert_eq!(refs[1].links.len(), 0);
        assert_eq!(refs[1].inspire_record_id, Some("8888888".to_string()));
    }

    #[test]
    fn test_convert_hit_with_missing_optional_fields() {
        let json = r#"{
            "hits": {
                "hits": [{
                    "metadata": {
                        "titles": [{"title": "Minimal Paper"}]
                    }
                }]
            }
        }"#;
        let resp: InspireResponse = serde_json::from_str(json).unwrap();
        let paper = InspireHepClient::convert_hit_to_paper(&resp.hits.hits[0]);

        assert_eq!(paper.title, "Minimal Paper");
        assert!(paper.authors.is_empty());
        assert!(paper.summary.is_empty());
        assert!(paper.id.is_empty());
        assert!(paper.doi.is_none());
        assert!(paper.inspire_id.is_none());
        assert!(paper.citation_count.is_none());
        assert!(paper.published.is_empty());
    }

    #[test]
    fn test_empty_references() {
        let meta = InspireMetadata {
            titles: None,
            authors: None,
            abstracts: None,
            arxiv_eprints: None,
            dois: None,
            citation_count: None,
            references: None,
            earliest_date: None,
        };
        let refs = InspireHepClient::convert_references(&meta);
        assert!(refs.is_empty());
    }

    #[tokio::test]
    async fn test_inspirehep_fetch_paper_published() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let response_json = r#"{
            "hits": {
                "hits": [{
                    "id": 1234567,
                    "metadata": {
                        "titles": [{"title": "Date Test Paper"}],
                        "arxiv_eprints": [{"value": "2301.12345"}],
                        "earliest_date": "2023-01-15"
                    }
                }]
            }
        }"#;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_json))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let source = InspireHepClient::new(client)
            .with_base_url(mock_server.uri())
            .with_rate_limit(std::time::Duration::from_millis(0));

        let paper = source.fetch_paper("2301.12345").await.unwrap();
        assert_eq!(paper.published, "2023-01-15");
        assert_eq!(paper.id, "2301.12345");
    }
}
