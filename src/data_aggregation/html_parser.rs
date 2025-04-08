use std::{thread, time};

use reqwest;
use scraper::Html;

pub struct ArxivHTMLDownloader {
    burst_limit: i32,
    current_request_number: i32,
    sleep_timer: time::Duration,
}

impl ArxivHTMLDownloader {
    pub fn new() -> ArxivHTMLDownloader {
        ArxivHTMLDownloader {
            burst_limit: 4,
            current_request_number: 0,
            sleep_timer: time::Duration::from_secs(1),
        }
    }

    pub fn request_html(mut self, html_url: &str) -> Html {
        Self::rate_limit_check(&mut self);
        parse_html(html_url)
    }

    pub fn rate_limit_check(&mut self) {
        if self.current_request_number == self.burst_limit {
            println!("Arxiv API rate limit exceeded. Sleep 1s...");
            thread::sleep(self.sleep_timer);
            self.current_request_number = 0
        }
        self.current_request_number += 1
    }
}
pub fn parse_html(html_url: &str) -> Html {
    let html_string = download_html(html_url)
        .inspect_err(|error| println!("WARNING: HTML download failed: {}", error))
        .unwrap_or_default();
    Html::parse_document(&html_string)
}

#[tokio::main]
async fn download_html(html_url: &str) -> Result<String, reqwest::Error> {
    let body: String = reqwest::get(html_url).await?.text().await?;
    Ok(body)
}
