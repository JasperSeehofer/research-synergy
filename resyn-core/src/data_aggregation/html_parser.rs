use scraper::Html;
use std::time::Instant;
use tokio::time::{Duration, sleep};
use tracing::{debug, warn};

use crate::error::ResynError;

pub struct ArxivHTMLDownloader {
    last_called: Option<Instant>,
    call_per_duration: Duration,
    client: reqwest::Client,
}

impl ArxivHTMLDownloader {
    pub fn new(client: reqwest::Client) -> ArxivHTMLDownloader {
        ArxivHTMLDownloader {
            last_called: None,
            call_per_duration: Duration::from_secs(3),
            client,
        }
    }

    pub fn with_rate_limit(mut self, duration: Duration) -> Self {
        self.call_per_duration = duration;
        self
    }

    pub async fn rate_limit_check(&mut self) {
        let now = Instant::now();

        if let Some(last_call) = self.last_called {
            let elapsed = now.duration_since(last_call);
            if elapsed < self.call_per_duration {
                let remaining = self.call_per_duration - elapsed;
                debug!("Rate limit: sleeping for {:?}", remaining);
                sleep(remaining).await;
            }
        }

        self.last_called = Some(Instant::now());
    }

    /// Download raw HTML string with rate limiting. Returns `String` (Send-safe)
    /// so callers can hold the result across further `.await` points before parsing.
    pub async fn download_raw(&mut self, html_url: &str) -> Result<String, ResynError> {
        self.rate_limit_check().await;
        self.download_html(html_url).await
    }

    pub async fn download_and_parse(&mut self, html_url: &str) -> Result<Html, ResynError> {
        let raw = self.download_raw(html_url).await?;
        Ok(Html::parse_document(&raw))
    }

    async fn download_html(&self, html_url: &str) -> Result<String, ResynError> {
        let body = self
            .client
            .get(html_url)
            .send()
            .await
            .map_err(|e| {
                warn!("HTML download failed for {html_url}: {e}");
                ResynError::HtmlDownload(format!("{html_url}: {e}"))
            })?
            .text()
            .await
            .map_err(|e| {
                warn!("Failed to read HTML body from {html_url}: {e}");
                ResynError::HtmlDownload(format!("{html_url}: {e}"))
            })?;
        Ok(body)
    }
}
