use anyhow::Result;
use std::{fs::File, io::Write};

use lopdf::Document;
use reqwest::{self, Error};

pub fn parse_pdf(pdf_url: &str) -> Vec<String> {
    let content = download_pdf(pdf_url);
    let document = Document::load_mem(&content.unwrap_or_default())
        .inspect(|x| println!("Loaded PDF: {:?}", x))
        .inspect_err(|x| println!("Error loading PDF: {:?}", x))
        .unwrap_or_default();
    let pages = document.get_pages();
    let mut document_text: Vec<String> = Vec::new();
    for (i, _) in pages.iter().enumerate() {
        let page_number = (i + 1) as u32;
        let text = document.extract_text(&[page_number]);
        document_text.push(text.unwrap_or_default());
    }
    document_text
}

#[tokio::main]
async fn download_pdf(pdf_url: &str) -> Result<Vec<u8>, Error> {
    let response = reqwest::get(pdf_url)
        .await
        .inspect(|x| println!("Response: {:?}", x));
    let content = match response.expect("Response failed.").bytes().await {
        Ok(content) => content,
        Err(e) => return Err(e),
    };

    let mut dest = File::create("./saved_papers/paper.pdf").expect("File not created.");
    let _ = dest.write_all(&content);
    Ok(Vec::from(content))
}
