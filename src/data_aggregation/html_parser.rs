use std::{thread::sleep, time::Duration};

use reqwest;
use scraper::Html;
use std::time::Instant;

pub struct ArxivHTMLDownloader {
    last_called: Option<Instant>,
    call_per_duration: Duration,
}

impl ArxivHTMLDownloader {
    pub fn new() -> ArxivHTMLDownloader {
        ArxivHTMLDownloader {
            last_called: None,
            call_per_duration: Duration::from_secs(3),
        }
    }

    pub fn request_html(mut self, html_url: &str) -> Html {
        Self::rate_limit_check(&mut self);
        parse_html(html_url)
    }

    pub fn rate_limit_check(&mut self) {
        let now = Instant::now();

        if let Some(last_call) = self.last_called {
            let elapsed = now.duration_since(last_call);
            if elapsed < self.call_per_duration {
                let remaining = self.call_per_duration - elapsed;
                println!("Rate limit exceeded. Sleep for {:?}", remaining);
                sleep(remaining);
            }
        }
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
