use anyhow::Result;
use reqwest;
use scraper::{ElementRef, Html, Selector};

use crate::datamodels::paper::{Link, Reference};

pub fn parse_html(html_url: &str) -> Html {
    let html_string = download_html(html_url).unwrap_or_default();
    Html::parse_document(&html_string)
}

#[tokio::main]
async fn download_html(html_url: &str) -> Result<String, reqwest::Error> {
    let body: String = reqwest::get(html_url).await?.text().await?;
    Ok(body)
}

pub fn get_references_for_arxiv_paper(html_url: &str) -> Vec<Reference> {
    let html_content = parse_html(html_url);
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
        println!("{}", referece_string);
        let (author, title) = trim_and_split_arxiv_reference_string(&referece_string);
        println!("Author: {}", author);
        println!("Title: {}", title);
        references.push(Reference {
            author,
            title,
            links: links.iter().map(|x| Link::from_url(x)).collect(),
        });
    }
    for reference in references {
        println!("{}", reference);
    }
    Vec::new()
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
