use crate::data_aggregation::html_parser::{parse_html, ArxivHTMLDownloader};
use crate::paper::{Link, Paper, Reference};
use scraper::{ElementRef, Selector};
use std::collections::HashSet;

use super::arxiv_api::get_paper_by_id;

pub fn aggregate_references_for_arxiv_paper(
    paper: &mut Paper,
    downloader: &mut ArxivHTMLDownloader,
) {
    downloader.rate_limit_check();
    let html_content = parse_html(&convert_pdf_url_to_html_url(&paper.pdf_url));
    let mut references: Vec<Reference> = Vec::new();
    let reference_selector = Selector::parse(r#"span[class="ltx_bibblock"]"#).unwrap();
    let references_elements = html_content.select(&reference_selector);
    for reference in references_elements {
        let mut links: Vec<String> = Vec::new();

        let mut reference_string = String::new();
        let mut em_title = String::new();
        for child in reference.children() {
            if let Some(text) = child.value().as_text() {
                reference_string.push_str(text);
            } else if let Some(element) = child.value().as_element() {
                if let Some(child_element) = ElementRef::wrap(child) {
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
        }
        let (author, title) = if !em_title.is_empty() {
            // Use <em> tag content as the title (more reliable than comma splitting)
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
        });
    }
    paper.references = references;
}

fn convert_pdf_url_to_html_url(pdf_url: &str) -> String {
    pdf_url.replace(".pdf", "").replace("pdf", "html")
}

fn strip_version_suffix(id: &str) -> String {
    if let Some(pos) = id.rfind('v') {
        if id[pos + 1..].chars().all(|c| c.is_ascii_digit()) && pos + 1 < id.len() {
            return id[..pos].to_string();
        }
    }
    id.to_string()
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

pub fn recursive_paper_search_by_references(paper_id: &str, max_depth: usize) -> Vec<Paper> {
    let mut visited_papers = HashSet::new();
    let mut papers: Vec<Paper> = Vec::new();
    let mut depth = 1;
    let mut referenced_paper_ids: Vec<String> = Vec::new();
    let mut new_referenced_papers: Vec<String> = Vec::new();
    let mut arxiv_html_downloader = ArxivHTMLDownloader::new();
    referenced_paper_ids.push(paper_id.to_string());

    while depth <= max_depth {
        println!("Current depth: {}", depth);
        println!(
            "{} referenced papers on this level",
            referenced_paper_ids.len()
        );
        for paper_id in &referenced_paper_ids {
            let paper_id = strip_version_suffix(paper_id);
            if visited_papers.contains(&paper_id) {
                println!(
                    "Already encountered paper with id {}. Continue...",
                    paper_id
                );
                continue;
            }
            visited_papers.insert(paper_id.clone());

            arxiv_html_downloader.rate_limit_check();
            let fetched = get_paper_by_id(&paper_id);
            let mut paper = match fetched {
                Ok(arxiv_paper) => Paper::from_arxiv_paper(&arxiv_paper),
                Err(e) => {
                    println!("WARNING: Failed to fetch paper {}: {}", paper_id, e);
                    continue;
                }
            };

            aggregate_references_for_arxiv_paper(&mut paper, &mut arxiv_html_downloader);
            println!("Paper has {} references.", paper.references.len());
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
    fn test_strip_version_suffix() {
        assert_eq!(strip_version_suffix("2301.12345v2"), "2301.12345");
        assert_eq!(strip_version_suffix("2301.12345v12"), "2301.12345");
        assert_eq!(strip_version_suffix("2301.12345"), "2301.12345");
        assert_eq!(strip_version_suffix("hep-ph/0601234v1"), "hep-ph/0601234");
    }
}
