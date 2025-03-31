use std::fmt;

use arxiv::Arxiv;

pub struct Paper {
    title: String,
    authors: Vec<String>,
    summary: String,
    id: String,
    last_updated: String,
    published: String,
    pdf_url: String,
    comment: Option<String>,
}

impl Paper {
    pub fn new() -> Paper {
        Paper {
            title: String::new(),
            authors: Vec::new(),
            summary: String::new(),
            id: String::new(),
            last_updated: String::new(),
            published: String::new(),
            pdf_url: String::new(),
            comment: Option::default(),
        }
    }
    pub fn from_arxiv_paper(arxiv_paper: &Arxiv) -> Paper {
        Paper {
            title: arxiv_paper.title.clone(),
            authors: arxiv_paper.authors.clone(),
            summary: arxiv_paper.summary.clone(),
            id: arxiv_paper.id.clone(),
            last_updated: arxiv_paper.updated.clone(),
            published: arxiv_paper.published.clone(),
            pdf_url: arxiv_paper.pdf_url.clone(),
            comment: arxiv_paper.comment.clone(),
        }
    }
}
impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shortened_summary: String = self.summary.clone();
        let _ = shortened_summary.split_off(50);
        shortened_summary.push_str("...");
        write!(
            f,
            " __________________\n/_| PAPER\n| Title: {}\n| Authors: {}\n| Summary: {}\n| ID: {}\n| Last updated: {}\n| Published: {}\n| PDF URL: {}\n| Comment: {}\n|_______________",
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
