use std::path::Iter;

use anyhow::Result;
use reqwest;
use scraper::{html, Html, Selector};

use crate::datamodels::paper::Reference;

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
    let reference_selector = Selector::parse(r#"span[class="ltx_bibblock"]"#).unwrap();
    let references = html_content.select(&reference_selector);
    for reference in references {
        let text = reference.text().collect::<Vec<_>>();
        println!("Reference: {:?}", reference);
        for textfield in text {
            println!("Textfield: {}", textfield);
        }
    }
    Vec::new()
}
