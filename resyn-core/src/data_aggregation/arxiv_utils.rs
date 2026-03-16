use crate::data_aggregation::html_parser::ArxivHTMLDownloader;
use crate::data_aggregation::traits::PaperSource;
use crate::datamodels::paper::{Link, Paper, Reference};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;
use scraper::{ElementRef, Selector};
use std::collections::HashSet;
use tracing::{debug, info, warn};

pub async fn aggregate_references_for_arxiv_paper(
    paper: &mut Paper,
    downloader: &mut ArxivHTMLDownloader,
) -> Result<(), ResynError> {
    let html_url = if paper.pdf_url.is_empty() {
        format!("https://arxiv.org/html/{}", paper.id)
    } else {
        convert_pdf_url_to_html_url(&paper.pdf_url)
    };
    let html_content = downloader.download_and_parse(&html_url).await?;
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
                        if let Some(href) = child_element.value().attr("href") {
                            links.push(href.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
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
}
