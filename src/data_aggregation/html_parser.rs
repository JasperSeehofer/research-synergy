use reqwest;
use scraper::Html;

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
