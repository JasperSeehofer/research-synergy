use arxiv::Arxiv;
use std::fmt::{self, Display};

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
            references: Vec::new(),
        }
    }
    pub fn from_arxiv_paper(arxiv_paper: &Arxiv) -> Paper {
        Paper {
            title: arxiv_paper.title.clone(),
            authors: arxiv_paper.authors.clone(),
            summary: arxiv_paper.summary.clone(),
            id: arxiv_paper.id.split("/").last().unwrap().to_string(),
            last_updated: arxiv_paper.updated.clone(),
            published: arxiv_paper.published.clone(),
            pdf_url: arxiv_paper.pdf_url.clone(),
            comment: arxiv_paper.comment.clone(),
            references: Vec::new(),
        }
    }
    pub fn get_arxiv_references_ids(&self) -> Vec<String> {
        self.references
            .iter()
            .map(|x| x.get_arxiv_id().unwrap_or(String::new()))
            .filter(|x| !x.is_empty())
            .collect()
    }
}
impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shortened_summary: String = self.summary.clone();
        let _ = shortened_summary.split_off(50);
        shortened_summary.push_str("...");
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

pub struct Reference {
    pub author: String,
    pub title: String,
    pub links: Vec<Link>,
}

impl Reference {
    pub fn new() -> Reference {
        Reference {
            author: String::new(),
            title: String::new(),
            links: Vec::new(),
        }
    }
    pub fn get_arxiv_id(&self) -> Result<String, ()> {
        let link: Option<&Link> = self
            .links
            .iter()
            .find(|link| matches!(link.journal, Journal::Arxiv));
        match link {
            Some(existing_link) => Ok(existing_link.url.split("/").last().unwrap().to_string()),
            None => Err(()),
        }
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

pub struct Link {
    url: String,
    journal: Journal,
}

impl Link {
    pub fn new() -> Link {
        Link {
            url: String::new(),
            journal: Journal::new(),
        }
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

pub enum Journal {
    Arxiv,
    Nature,
    PhysRev,
    Unkown,
}

impl Journal {
    pub fn new() -> Journal {
        Journal::Unkown
    }
    pub fn from_url(url: &str) -> Journal {
        if url.contains("arxiv") {
            Journal::Arxiv
        } else {
            Journal::Unkown
        }
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Journal::Arxiv => write!(f, "arXiv"),
            Journal::Nature => write!(f, "Nature"),
            Journal::PhysRev => write!(f, "Phys. Rev."),
            Journal::Unkown => write!(f, "unknown"),
        }
    }
}
