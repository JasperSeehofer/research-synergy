use crate::data_aggregation::html_parser::ArxivHTMLDownloader;
use crate::data_aggregation::traits::PaperSource;
use crate::datamodels::paper::{Link, Paper, Reference};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;
use regex::Regex;
use scraper::{ElementRef, Selector};
use std::collections::HashSet;
use std::sync::OnceLock;
use tracing::{debug, info, warn};

static ARXIV_NEW_RE: OnceLock<Regex> = OnceLock::new();
static ARXIV_OLD_RE: OnceLock<Regex> = OnceLock::new();
static DOI_RE: OnceLock<Regex> = OnceLock::new();

fn arxiv_new_re() -> &'static Regex {
    ARXIV_NEW_RE.get_or_init(|| {
        Regex::new(r"\b((?:0[0-9]|1[0-9]|2[0-9])\d{2}\.\d{4,5}(?:v\d+)?)\b").unwrap()
    })
}

fn arxiv_old_re() -> &'static Regex {
    ARXIV_OLD_RE.get_or_init(|| Regex::new(r"\b([a-zA-Z][a-zA-Z0-9\-]+/\d{7}(?:v\d+)?)\b").unwrap())
}

fn doi_re() -> &'static Regex {
    DOI_RE.get_or_init(|| Regex::new(r"\b(10\.\d{4,}/[^\s,;)\]]+)").unwrap())
}

pub async fn aggregate_references_for_arxiv_paper(
    paper: &mut Paper,
    downloader: &mut ArxivHTMLDownloader,
) -> Result<(), ResynError> {
    let primary_url = if paper.pdf_url.is_empty() {
        format!("https://arxiv.org/html/{}", paper.id)
    } else {
        convert_pdf_url_to_html_url(&paper.pdf_url)
    };
    // ar5iv fallback: arxiv.org/html/ only serves papers from ~2023+; older papers
    // (and 2015–2022 papers that cite pre-2015 work with arXiv IDs) are on ar5iv.
    // Download raw HTML as String (Send-safe) before parsing to Html (!Send),
    // so the fallback await does not hold a non-Send value across an await point.
    let raw_html = match downloader.download_raw(&primary_url).await {
        Ok(s) => s,
        Err(e) => {
            let fallback_url = format!("https://ar5iv.labs.arxiv.org/html/{}", paper.id);
            debug!(
                paper_id = paper.id.as_str(),
                error = %e,
                fallback = fallback_url.as_str(),
                "Primary HTML unavailable, trying ar5iv fallback"
            );
            downloader.download_raw(&fallback_url).await?
        }
    };
    let html_content = scraper::Html::parse_document(&raw_html);
    let mut references: Vec<Reference> = Vec::new();
    let reference_selector =
        Selector::parse(r#"span[class="ltx_bibblock"]"#).expect("static CSS selector is valid");
    let references_elements = html_content.select(&reference_selector);
    for reference in references_elements {
        let mut links: Vec<String> = Vec::new();

        let mut reference_string = String::new();
        let mut em_title = String::new();
        for child in reference.children() {
            if let Some(text) = child.value().as_text() {
                reference_string.push_str(text);
            } else if let Some(element) = child.value().as_element()
                && let Some(child_element) = ElementRef::wrap(child)
            {
                match element.name() {
                    "em" => {
                        let text = child_element.text().collect::<String>();
                        em_title = text.trim().to_string();
                        reference_string.push_str(&em_title);
                    }
                    "a" => {
                        // Always include link text (arxiv.org/html uses <a> without href for DOIs)
                        let text = child_element.text().collect::<String>();
                        reference_string.push_str(&text);
                        if let Some(href) = child_element.value().attr("href") {
                            links.push(href.to_string());
                        }
                    }
                    _ => {
                        // Collect text from spans and other inline elements
                        // (arxiv.org/html uses <span> instead of <em> for italic titles)
                        let text = child_element.text().collect::<String>();
                        reference_string.push_str(&text);
                    }
                }
            }
        }
        // --- Text-based ID extraction (per D-01, D-02, D-03) ---

        // Build dedup set from existing <a>-tag links (D-03: merge, don't duplicate)
        let mut seen_arxiv_ids: HashSet<String> = links
            .iter()
            .filter_map(|url| {
                if url.contains("arxiv") {
                    url.split('/').next_back().map(strip_version_suffix)
                } else {
                    None
                }
            })
            .collect();

        // Extract new-format arXiv IDs from plain text (per D-01)
        let mut text_extracted_eprint: Option<String> = None;
        for cap in arxiv_new_re().captures_iter(&reference_string) {
            let id = strip_version_suffix(&cap[1]);
            if seen_arxiv_ids.insert(id.clone()) {
                links.push(format!("https://arxiv.org/abs/{}", id));
                if text_extracted_eprint.is_none() {
                    text_extracted_eprint = Some(id);
                }
            }
        }

        // Extract old-format arXiv IDs from plain text (per D-01)
        for cap in arxiv_old_re().captures_iter(&reference_string) {
            let id = strip_version_suffix(&cap[1]);
            if seen_arxiv_ids.insert(id.clone()) {
                links.push(format!("https://arxiv.org/abs/{}", id));
                if text_extracted_eprint.is_none() {
                    text_extracted_eprint = Some(id);
                }
            }
        }

        // Extract DOI from plain text (per D-02)
        let text_extracted_doi = doi_re().captures(&reference_string).map(|cap| {
            cap[1]
                .trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        });

        let (author, title) = if !em_title.is_empty() {
            let author = reference_string
                .split(&em_title)
                .next()
                .unwrap_or_default()
                .trim()
                .trim_end_matches(',')
                .trim()
                .to_string();
            (author, em_title)
        } else {
            trim_and_split_arxiv_reference_string(&reference_string)
        };
        references.push(Reference {
            author,
            title,
            links: links.iter().map(|x| Link::from_url(x)).collect(),
            doi: text_extracted_doi,
            arxiv_eprint: text_extracted_eprint,
            ..Default::default()
        });
    }
    paper.references = references;
    Ok(())
}

fn convert_pdf_url_to_html_url(pdf_url: &str) -> String {
    pdf_url.replace(".pdf", "").replace("pdf", "html")
}

fn trim_and_split_arxiv_reference_string(text: &str) -> (String, String) {
    let mut trimmed_text: Vec<&str> = text
        .trim()
        .trim_end_matches(".")
        .split(",")
        .map(|x| x.trim())
        .filter(|x| !x.trim().is_empty())
        .collect();
    let title = trimmed_text.pop().unwrap_or_default();
    let author = trimmed_text.join(", ");
    (author.trim().to_string(), title.trim().to_string())
}

pub async fn recursive_paper_search_by_references(
    paper_id: &str,
    max_depth: usize,
    source: &mut dyn PaperSource,
) -> Vec<Paper> {
    let mut visited_papers = HashSet::new();
    let mut papers: Vec<Paper> = Vec::new();
    let mut depth = 1;
    let mut referenced_paper_ids: Vec<String> = vec![paper_id.to_string()];
    let mut new_referenced_papers: Vec<String> = Vec::new();

    while depth <= max_depth {
        info!(
            depth,
            count = referenced_paper_ids.len(),
            "Processing depth level"
        );
        for paper_id in &referenced_paper_ids {
            let paper_id = strip_version_suffix(paper_id);
            if visited_papers.contains(&paper_id) {
                debug!(paper_id, "Skipping already visited paper");
                continue;
            }
            visited_papers.insert(paper_id.clone());

            let mut paper = match source.fetch_paper(&paper_id).await {
                Ok(p) => p,
                Err(e) => {
                    warn!(paper_id, error = %e, "Failed to fetch paper");
                    continue;
                }
            };

            if let Err(e) = source.fetch_references(&mut paper).await {
                warn!(paper_id, error = %e, "Failed to fetch references");
            }
            info!(
                paper_id,
                reference_count = paper.references.len(),
                "Paper processed"
            );
            let mut arxiv_paper_ids: Vec<String> = paper.get_arxiv_references_ids();
            new_referenced_papers.append(&mut arxiv_paper_ids);
            papers.push(paper);
        }
        referenced_paper_ids = new_referenced_papers.clone();
        new_referenced_papers = Vec::new();
        depth += 1;
    }
    papers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_and_split_arxiv_reference_string() {
        let test_text = "Author 1, Author 2, Title, .";
        let (author, title) = trim_and_split_arxiv_reference_string(test_text);
        assert_eq!("Author 1, Author 2", &author);
        assert_eq!("Title", &title);

        let test_text_2 = "Author, Title, , .";
        let (author, title) = trim_and_split_arxiv_reference_string(test_text_2);
        assert_eq!("Author", author);
        assert_eq!("Title", title);
    }

    #[test]
    fn test_trim_and_split_empty_string() {
        let (author, title) = trim_and_split_arxiv_reference_string("");
        assert_eq!(author, "");
        assert_eq!(title, "");
    }

    #[test]
    fn test_trim_and_split_single_item() {
        let (author, title) = trim_and_split_arxiv_reference_string("Just a Title.");
        assert_eq!(author, "");
        assert_eq!(title, "Just a Title");
    }

    #[test]
    fn test_convert_pdf_url_to_html_url() {
        assert_eq!(
            convert_pdf_url_to_html_url("https://arxiv.org/pdf/2301.12345.pdf"),
            "https://arxiv.org/html/2301.12345"
        );
        assert_eq!(
            convert_pdf_url_to_html_url("https://arxiv.org/pdf/2301.12345"),
            "https://arxiv.org/html/2301.12345"
        );
    }

    #[test]
    fn test_convert_pdf_url_to_html_url_empty_returns_empty() {
        // convert_pdf_url_to_html_url itself has no guard — empty in, empty out
        assert_eq!(convert_pdf_url_to_html_url(""), "");
    }

    #[test]
    fn test_html_url_fallback_for_empty_pdf_url() {
        // Validates the fallback logic used in aggregate_references_for_arxiv_paper
        let paper_id = "2503.18887";
        let pdf_url = "";
        let html_url = if pdf_url.is_empty() {
            format!("https://arxiv.org/html/{}", paper_id)
        } else {
            convert_pdf_url_to_html_url(pdf_url)
        };
        assert_eq!(html_url, "https://arxiv.org/html/2503.18887");
    }

    #[test]
    fn test_html_url_uses_pdf_url_when_non_empty() {
        let paper_id = "2503.18887";
        let pdf_url = "https://arxiv.org/pdf/2503.18887.pdf";
        let html_url = if pdf_url.is_empty() {
            format!("https://arxiv.org/html/{}", paper_id)
        } else {
            convert_pdf_url_to_html_url(pdf_url)
        };
        assert_eq!(html_url, "https://arxiv.org/html/2503.18887");
    }

    #[test]
    fn test_arxiv_new_re_matches() {
        let re = super::arxiv_new_re();
        assert!(re.is_match("arXiv:2301.12345"));
        assert!(re.is_match("arXiv:2301.12345v2"));
        assert!(re.is_match("some text 0912.1234 more text"));
        // Should NOT match short numbers
        assert!(!re.is_match("91.1234"));
    }

    #[test]
    fn test_arxiv_old_re_matches() {
        let re = super::arxiv_old_re();
        assert!(re.is_match("hep-ph/0601234"));
        assert!(re.is_match("astro-ph/9912345v1"));
        assert!(!re.is_match("12345/0601234"));
    }

    #[test]
    fn test_doi_re_matches() {
        let re = super::doi_re();
        let cap = re.captures("10.1103/PhysRevD.107.012345").unwrap();
        assert_eq!(&cap[1], "10.1103/PhysRevD.107.012345");
    }
}
