#[cfg(feature = "ssr")]
use arxiv::Arxiv;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use crate::error::ResynError;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum DataSource {
    #[default]
    Arxiv,
    InspireHep,
    Merged,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Paper {
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub id: String,
    pub last_updated: String,
    pub published: String,
    pub pdf_url: String,
    pub comment: Option<String>,
    pub references: Vec<Reference>,
    pub doi: Option<String>,
    pub inspire_id: Option<String>,
    pub citation_count: Option<u32>,
    pub source: DataSource,
}

impl Paper {
    pub fn new() -> Paper {
        Paper::default()
    }

    #[cfg(feature = "ssr")]
    pub fn from_arxiv_paper(arxiv_paper: &Arxiv) -> Result<Paper, ResynError> {
        let id = arxiv_paper
            .id
            .split("/")
            .last()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ResynError::InvalidPaperId(arxiv_paper.id.clone()))?
            .to_string();

        Ok(Paper {
            title: arxiv_paper.title.clone(),
            authors: arxiv_paper.authors.clone(),
            summary: arxiv_paper.summary.clone(),
            id,
            last_updated: arxiv_paper.updated.clone(),
            published: arxiv_paper.published.clone(),
            pdf_url: arxiv_paper.pdf_url.clone(),
            comment: arxiv_paper.comment.clone(),
            references: Vec::new(),
            ..Default::default()
        })
    }

    pub fn get_arxiv_references_ids(&self) -> Vec<String> {
        self.references
            .iter()
            .filter_map(|r| r.get_arxiv_id().ok())
            .filter(|id| {
                if id.is_empty() {
                    tracing::warn!(
                        "Skipping empty arXiv ID in references for paper {}",
                        self.id
                    );
                    false
                } else {
                    true
                }
            })
            .collect()
    }
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let shortened_summary: String = if self.summary.len() > 50 {
            let mut s = self.summary[..50].to_string();
            s.push_str("...");
            s
        } else {
            self.summary.clone()
        };
        write!(
            f,
            "___________________\n| PAPER\n| Title: {}\n| Authors: {}\n| Summary: {}\n| ID: {}\n| Last updated: {}\n| Published: {}\n| PDF URL: {}\n| Comment: {}\n|_______________",
            self.title.replace("\n", ""),
            self.authors.join(", "),
            shortened_summary,
            self.id,
            self.last_updated,
            self.published,
            self.pdf_url,
            self.comment.clone().unwrap_or_default(),
        )
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Reference {
    pub author: String,
    pub title: String,
    pub links: Vec<Link>,
    pub doi: Option<String>,
    pub arxiv_eprint: Option<String>,
    pub inspire_record_id: Option<String>,
    pub label: Option<String>,
}

impl Reference {
    pub fn new() -> Reference {
        Reference::default()
    }

    pub fn get_arxiv_id(&self) -> Result<String, ResynError> {
        // Primary: check Links for a Journal::Arxiv entry
        if let Some(link) = self
            .links
            .iter()
            .find(|l| matches!(l.journal, Journal::Arxiv))
        {
            return link
                .url
                .split('/')
                .next_back()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .ok_or(ResynError::NoArxivLink);
        }
        // Fallback: check arxiv_eprint field (populated by text extraction)
        self.arxiv_eprint.clone().ok_or(ResynError::NoArxivLink)
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Author: {}", self.author)?;
        writeln!(f, "Title: {}", self.title)?;

        if !self.links.is_empty() {
            writeln!(f, "Links:")?;
            for link in &self.links {
                writeln!(f, "- {}", link)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub journal: Journal,
}

impl Link {
    pub fn new() -> Link {
        Link::default()
    }
    pub fn from_url(url: &str) -> Link {
        Link {
            url: url.to_string(),
            journal: Journal::from_url(url),
        }
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.url, self.journal)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum Journal {
    Arxiv,
    Nature,
    PhysRev,
    #[default]
    Unknown,
}

impl Journal {
    pub fn from_url(url: &str) -> Journal {
        if url.contains("arxiv") {
            Journal::Arxiv
        } else {
            Journal::Unknown
        }
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Journal::Arxiv => write!(f, "arXiv"),
            Journal::Nature => write!(f, "Nature"),
            Journal::PhysRev => write!(f, "Phys. Rev."),
            Journal::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ssr")]
    #[test]
    fn test_from_arxiv_paper() {
        let arxiv = Arxiv {
            id: "http://arxiv.org/abs/2301.12345v1".to_string(),
            title: "Test Paper".to_string(),
            authors: vec!["Author One".to_string()],
            summary: "A summary".to_string(),
            updated: "2023-01-01".to_string(),
            published: "2023-01-01".to_string(),
            pdf_url: "http://arxiv.org/pdf/2301.12345v1".to_string(),
            comment: Some("10 pages".to_string()),
        };
        let paper = Paper::from_arxiv_paper(&arxiv).unwrap();
        assert_eq!(paper.id, "2301.12345v1");
        assert_eq!(paper.title, "Test Paper");
        assert_eq!(paper.authors, vec!["Author One"]);
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_from_arxiv_paper_empty_id() {
        let arxiv = Arxiv {
            id: "".to_string(),
            title: "Test".to_string(),
            authors: vec![],
            summary: "".to_string(),
            updated: "".to_string(),
            published: "".to_string(),
            pdf_url: "".to_string(),
            comment: None,
        };
        assert!(Paper::from_arxiv_paper(&arxiv).is_err());
    }

    #[test]
    fn test_get_arxiv_references_ids() {
        let mut paper = Paper::new();
        paper.references = vec![
            Reference {
                author: "Author".to_string(),
                title: "Title".to_string(),
                links: vec![Link::from_url("https://arxiv.org/abs/2301.11111")],
                ..Default::default()
            },
            Reference {
                author: "Author2".to_string(),
                title: "Title2".to_string(),
                links: vec![Link::from_url("https://nature.com/article/123")],
                ..Default::default()
            },
            Reference {
                author: "Author3".to_string(),
                title: "Title3".to_string(),
                links: vec![Link::from_url("https://arxiv.org/abs/2301.22222")],
                ..Default::default()
            },
        ];
        let ids = paper.get_arxiv_references_ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "2301.11111");
        assert_eq!(ids[1], "2301.22222");
    }

    #[test]
    fn test_get_arxiv_id_no_arxiv_link() {
        let reference = Reference {
            author: "Author".to_string(),
            title: "Title".to_string(),
            links: vec![Link::from_url("https://nature.com/article/123")],
            ..Default::default()
        };
        assert!(reference.get_arxiv_id().is_err());
    }

    #[test]
    fn test_link_from_url() {
        let arxiv_link = Link::from_url("https://arxiv.org/abs/2301.12345");
        assert!(matches!(arxiv_link.journal, Journal::Arxiv));
        assert_eq!(arxiv_link.url, "https://arxiv.org/abs/2301.12345");

        let other_link = Link::from_url("https://nature.com/article/123");
        assert!(matches!(other_link.journal, Journal::Unknown));
    }

    #[test]
    fn test_paper_display_short_summary() {
        let mut paper = Paper::new();
        paper.title = "Test".to_string();
        paper.summary = "Short".to_string();
        let display = format!("{}", paper);
        assert!(display.contains("Short"));
    }

    #[test]
    fn test_get_arxiv_id_fallback_to_eprint() {
        let reference = Reference {
            author: "Author".to_string(),
            title: "Title".to_string(),
            links: vec![Link::from_url("https://nature.com/article/123")],
            arxiv_eprint: Some("2301.12345".to_string()),
            ..Default::default()
        };
        assert_eq!(reference.get_arxiv_id().unwrap(), "2301.12345");
    }

    #[test]
    fn test_get_arxiv_id_prefers_link_over_eprint() {
        let reference = Reference {
            author: "Author".to_string(),
            title: "Title".to_string(),
            links: vec![Link::from_url("https://arxiv.org/abs/2301.99999")],
            arxiv_eprint: Some("2301.11111".to_string()),
            ..Default::default()
        };
        // Link takes priority over eprint
        assert_eq!(reference.get_arxiv_id().unwrap(), "2301.99999");
    }

    #[test]
    fn test_get_arxiv_id_eprint_only_no_links() {
        let reference = Reference {
            author: "Author".to_string(),
            title: "Title".to_string(),
            links: vec![],
            arxiv_eprint: Some("hep-ph/0601234".to_string()),
            ..Default::default()
        };
        assert_eq!(reference.get_arxiv_id().unwrap(), "hep-ph/0601234");
    }

    #[test]
    fn test_get_arxiv_id_no_link_no_eprint() {
        let reference = Reference {
            author: "Author".to_string(),
            title: "Title".to_string(),
            links: vec![Link::from_url("https://nature.com/article/123")],
            arxiv_eprint: None,
            ..Default::default()
        };
        assert!(reference.get_arxiv_id().is_err());
    }

    #[test]
    fn test_get_arxiv_references_ids_filters_empty() {
        let mut paper = Paper::new();
        paper.id = "2301.00001".to_string();
        paper.references = vec![
            Reference {
                author: "Valid".to_string(),
                title: "Valid Paper".to_string(),
                links: vec![Link::from_url("https://arxiv.org/abs/2301.11111")],
                ..Default::default()
            },
            Reference {
                author: "Empty Eprint".to_string(),
                title: "No ArXiv ID".to_string(),
                links: vec![],
                arxiv_eprint: Some("".to_string()),
                ..Default::default()
            },
            Reference {
                author: "Another Valid".to_string(),
                title: "Another Paper".to_string(),
                links: vec![],
                arxiv_eprint: Some("2301.22222".to_string()),
                ..Default::default()
            },
        ];
        let ids = paper.get_arxiv_references_ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "2301.11111");
        assert_eq!(ids[1], "2301.22222");
    }

    #[test]
    fn test_get_arxiv_references_ids_filters_empty_link() {
        let mut paper = Paper::new();
        paper.id = "2301.00002".to_string();
        paper.references = vec![Reference {
            author: "Bad Link".to_string(),
            title: "Empty URL Segment".to_string(),
            links: vec![Link::from_url("https://arxiv.org/abs/")],
            ..Default::default()
        }];
        let ids = paper.get_arxiv_references_ids();
        assert!(ids.is_empty(), "Empty URL last segment should be filtered out");
    }
}
