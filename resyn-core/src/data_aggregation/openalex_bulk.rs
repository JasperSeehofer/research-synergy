use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

use crate::datamodels::paper::{DataSource, Paper};
use crate::error::ResynError;

const OPENALEX_API: &str = "https://api.openalex.org/works";

// Regex for extracting arXiv IDs from landing_page_url like "http://arxiv.org/abs/2303.08774"
const ARXIV_URL_PREFIX: &str = "arxiv.org/abs/";

#[derive(Debug, Deserialize)]
pub struct OpenAlexPage {
    pub results: Vec<OpenAlexWork>,
    pub meta: OpenAlexMeta,
}

#[derive(Debug, Deserialize)]
pub struct OpenAlexMeta {
    pub count: u64,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAlexWork {
    /// Full OpenAlex URL, e.g. "https://openalex.org/W2741809807"
    pub id: String,
    /// Journal DOI — may or may not be the arXiv 10.48550 form
    pub doi: Option<String>,
    pub title: Option<String>,
    pub display_name: Option<String>,
    pub publication_date: Option<String>,
    #[serde(default)]
    pub referenced_works: Vec<String>,
    #[serde(default)]
    pub authorships: Vec<OpenAlexAuthorship>,
    #[serde(default)]
    pub locations: Vec<OpenAlexLocation>,
    /// Token → list of positions; reconstruct with `reconstruct_abstract()`
    pub abstract_inverted_index: Option<HashMap<String, Vec<usize>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAlexLocation {
    pub landing_page_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAlexAuthorship {
    pub author: OpenAlexAuthor,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAlexAuthor {
    pub display_name: String,
}

impl OpenAlexWork {
    /// Extract the arXiv ID for this work, trying three sources in order:
    /// 1. DOI field in the "10.48550/arxiv.XXXX.YYYYY" form
    /// 2. Any location's landing_page_url containing "arxiv.org/abs/"
    pub fn arxiv_id(&self) -> Option<String> {
        // 1. Try 10.48550 DOI
        if let Some(doi) = &self.doi {
            let lower = doi.to_lowercase();
            let prefix = "10.48550/arxiv.";
            if let Some(pos) = lower.find(prefix) {
                return Some(doi[pos + prefix.len()..].to_string());
            }
        }

        // 2. Try location landing_page_url
        for loc in &self.locations {
            if let Some(url) = &loc.landing_page_url {
                if let Some(pos) = url.to_lowercase().find(ARXIV_URL_PREFIX) {
                    let raw = &url[pos + ARXIV_URL_PREFIX.len()..];
                    // Strip trailing slashes or query params
                    let id = raw.split(['/', '?', '#']).next().unwrap_or("").to_string();
                    if !id.is_empty() {
                        return Some(id);
                    }
                }
            }
        }

        None
    }

    /// Reconstruct the abstract from OpenAlex's inverted index (token → positions).
    pub fn reconstruct_abstract(&self) -> String {
        let Some(ref aii) = self.abstract_inverted_index else {
            return String::new();
        };
        if aii.is_empty() {
            return String::new();
        }
        // Collect (position, token) pairs, sort by position, join.
        let mut pairs: Vec<(usize, &str)> = aii
            .iter()
            .flat_map(|(token, positions)| positions.iter().map(move |&pos| (pos, token.as_str())))
            .collect();
        pairs.sort_unstable_by_key(|(pos, _)| *pos);
        pairs.into_iter().map(|(_, tok)| tok).collect::<Vec<_>>().join(" ")
    }

    pub fn to_paper(&self) -> Option<Paper> {
        let arxiv_id = self.arxiv_id()?;
        let title = self
            .title
            .clone()
            .or_else(|| self.display_name.clone())
            .unwrap_or_default();
        let authors: Vec<String> = self
            .authorships
            .iter()
            .map(|a| a.author.display_name.clone())
            .collect();
        let published = self.publication_date.clone().unwrap_or_default();
        let summary = self.reconstruct_abstract();
        Some(Paper {
            id: arxiv_id,
            title,
            authors,
            summary,
            last_updated: published.clone(),
            published,
            pdf_url: String::new(),
            comment: None,
            doi: self.doi.clone(),
            inspire_id: None,
            citation_count: None,
            source: DataSource::Arxiv,
            references: Vec::new(),
        })
    }
}

pub struct OpenAlexBulkLoader {
    client: Client,
    mailto: String,
}

impl OpenAlexBulkLoader {
    pub fn new(client: Client, mailto: impl Into<String>) -> Self {
        Self {
            client,
            mailto: mailto.into(),
        }
    }

    pub async fn fetch_page(&self, filter: &str, cursor: &str) -> Result<OpenAlexPage, ResynError> {
        let url = format!(
            "{}?filter={}&per-page=200&cursor={}&mailto={}",
            OPENALEX_API, filter, cursor, self.mailto
        );
        let page: OpenAlexPage = self
            .client
            .get(&url)
            .header(
                "User-Agent",
                format!("resyn/0.1 (mailto:{})", self.mailto),
            )
            .send()
            .await
            .map_err(|e| ResynError::OpenAlexApi(format!("request failed: {e}")))?
            .error_for_status()
            .map_err(|e| ResynError::OpenAlexApi(format!("API error: {e}")))?
            .json()
            .await
            .map_err(|e| ResynError::OpenAlexApi(format!("JSON parse failed: {e}")))?;
        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_work(doi: Option<&str>, locations: Vec<&str>) -> OpenAlexWork {
        OpenAlexWork {
            id: "https://openalex.org/W123".to_string(),
            doi: doi.map(|s| s.to_string()),
            title: Some("Test".to_string()),
            display_name: None,
            publication_date: None,
            referenced_works: vec![],
            authorships: vec![],
            locations: locations
                .into_iter()
                .map(|url| OpenAlexLocation {
                    landing_page_url: Some(url.to_string()),
                })
                .collect(),
            abstract_inverted_index: None,
        }
    }

    #[test]
    fn test_arxiv_id_from_1048550_doi() {
        let w = make_work(
            Some("https://doi.org/10.48550/arxiv.2401.04191"),
            vec![],
        );
        assert_eq!(w.arxiv_id(), Some("2401.04191".to_string()));
    }

    #[test]
    fn test_arxiv_id_from_1048550_doi_mixed_case() {
        let w = make_work(Some("https://doi.org/10.48550/arXiv.2008.02217"), vec![]);
        assert_eq!(w.arxiv_id(), Some("2008.02217".to_string()));
    }

    #[test]
    fn test_arxiv_id_from_location_url() {
        let w = make_work(
            Some("https://doi.org/10.1016/j.jcp.2022.111402"),
            vec!["http://arxiv.org/abs/2202.11821"],
        );
        assert_eq!(w.arxiv_id(), Some("2202.11821".to_string()));
    }

    #[test]
    fn test_location_url_strips_version() {
        let w = make_work(None, vec!["https://arxiv.org/abs/1706.03762v5"]);
        // We return the raw segment including version; strip_version_suffix handles that upstream
        assert_eq!(w.arxiv_id(), Some("1706.03762v5".to_string()));
    }

    #[test]
    fn test_non_arxiv_returns_none() {
        let w = make_work(
            Some("https://doi.org/10.1103/physrevlett.123.456"),
            vec!["https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.123.456"],
        );
        assert_eq!(w.arxiv_id(), None);
    }

    #[test]
    fn test_to_paper_populates_fields() {
        let mut w = make_work(None, vec!["http://arxiv.org/abs/1706.03762"]);
        w.title = Some("Attention Is All You Need".to_string());
        w.publication_date = Some("2017-06-12".to_string());
        w.authorships = vec![OpenAlexAuthorship {
            author: OpenAlexAuthor {
                display_name: "Ashish Vaswani".to_string(),
            },
        }];
        let paper = w.to_paper().unwrap();
        assert_eq!(paper.id, "1706.03762");
        assert_eq!(paper.title, "Attention Is All You Need");
    }
}
