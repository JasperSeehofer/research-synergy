use research_synergy::data_aggregation::html_parser::ArxivHTMLDownloader;
use research_synergy::datamodels::paper::Paper;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn mock_arxiv_html_with_references() -> String {
    r#"<!DOCTYPE html>
    <html>
    <body>
    <span class="ltx_bibblock">Author One, <em>First Paper Title</em>,
        <a href="https://arxiv.org/abs/2301.11111">arXiv:2301.11111</a>
    </span>
    <span class="ltx_bibblock">Author Two, <em>Second Paper Title</em>,
        <a href="https://nature.com/article/123">Nature</a>
    </span>
    <span class="ltx_bibblock">Author Three, <em>Third Paper</em>,
        <a href="https://arxiv.org/abs/2301.22222">arXiv:2301.22222</a>
    </span>
    </body>
    </html>"#
        .to_string()
}

#[tokio::test]
async fn test_aggregate_references_from_mock_html() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/html/2301.99999"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_arxiv_html_with_references()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let mut downloader = ArxivHTMLDownloader::new(client).with_rate_limit(Duration::from_millis(0));

    let mut paper = Paper::new();
    paper.pdf_url = format!("{}/pdf/2301.99999", mock_server.uri());

    let result =
        research_synergy::data_aggregation::arxiv_utils::aggregate_references_for_arxiv_paper(
            &mut paper,
            &mut downloader,
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(paper.references.len(), 3);

    assert_eq!(paper.references[0].title, "First Paper Title");
    assert_eq!(paper.references[2].title, "Third Paper");

    let arxiv_ids = paper.get_arxiv_references_ids();
    assert_eq!(arxiv_ids.len(), 2);
    assert_eq!(arxiv_ids[0], "2301.11111");
    assert_eq!(arxiv_ids[1], "2301.22222");
}

#[tokio::test]
async fn test_aggregate_references_html_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/html/2301.00000"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let mut downloader = ArxivHTMLDownloader::new(client).with_rate_limit(Duration::from_millis(0));

    let mut paper = Paper::new();
    paper.pdf_url = format!("{}/pdf/2301.00000", mock_server.uri());

    // Should still succeed (404 returns HTML body which just has no references)
    let result =
        research_synergy::data_aggregation::arxiv_utils::aggregate_references_for_arxiv_paper(
            &mut paper,
            &mut downloader,
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(paper.references.len(), 0);
}

#[tokio::test]
async fn test_rate_limiter_enforces_delay() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/html/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html></html>"))
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let mut downloader =
        ArxivHTMLDownloader::new(client).with_rate_limit(Duration::from_millis(100));

    let start = std::time::Instant::now();
    let _ = downloader
        .download_and_parse(&format!("{}/html/test", mock_server.uri()))
        .await;
    let _ = downloader
        .download_and_parse(&format!("{}/html/test", mock_server.uri()))
        .await;
    let elapsed = start.elapsed();

    // Second call should have been delayed by at least 100ms
    assert!(
        elapsed >= Duration::from_millis(90),
        "Rate limiter should enforce delay, elapsed: {:?}",
        elapsed
    );
}
