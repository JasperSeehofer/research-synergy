use crate::data_aggregation::html_parser::parse_html;
use crate::paper::{Link, Paper, Reference};
use scraper::{ElementRef, Selector};
use std::collections::HashSet;

use super::arxiv_api::get_paper_by_id;

pub fn aggregate_references_for_arxiv_paper(paper: &mut Paper) {
    let html_content = parse_html(&convert_pdf_url_to_html_url(&paper.pdf_url));
    let mut references: Vec<Reference> = Vec::new();
    let reference_selector = Selector::parse(r#"span[class="ltx_bibblock"]"#).unwrap();
    let references_elements = html_content.select(&reference_selector);
    for reference in references_elements {
        let mut links: Vec<String> = Vec::new();

        let mut referece_string = String::new();
        for child in reference.children() {
            if let Some(text) = child.value().as_text() {
                referece_string.push_str(text);
            } else if let Some(element) = child.value().as_element() {
                if let Some(child_element) = ElementRef::wrap(child) {
                    match element.name() {
                        "em" => {
                            let text = child_element.text().collect::<String>();
                            let _ = &referece_string.push_str(&text);
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
        let (author, title) = trim_and_split_arxiv_reference_string(&referece_string);
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
    referenced_paper_ids.push(paper_id.to_string());

    while depth <= max_depth {
        println!("Current depth: {}", depth);
        println!(
            "{} referenced papers on this level",
            referenced_paper_ids.len()
        );
        for paper_id in referenced_paper_ids {
            if visited_papers.contains(&paper_id) {
                println!(
                    "Already encountered paper with id {}. Continue...",
                    paper_id
                );
            } else {
                visited_papers.insert(paper_id.clone());
            }

            let mut paper = Paper::from_arxiv_paper(&get_paper_by_id(&paper_id).unwrap());

            aggregate_references_for_arxiv_paper(&mut paper);
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
}
