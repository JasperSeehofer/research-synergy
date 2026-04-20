use resyn_core::data_aggregation::semantic_scholar_api::SemanticScholarSource;
use resyn_core::data_aggregation::traits::PaperSource;
use resyn_core::datamodels::paper::Paper;
use resyn_core::error::ResynError;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn paper_response() -> String {
    r#"{
        "title": "Epidemic spreading in scale-free networks",
        "authors": [{"name": "Pastor-Satorras, R."}, {"name": "Vespignani, A."}],
        "year": 2001,
        "abstract": "A study of epidemic spreading.",
        "externalIds": {"ArXiv": "cond-mat/0010317", "DOI": "10.1103/PhysRevLett.86.3200"},
        "citationCount": 4200,
        "publicationDate": "2001-04-02"
    }"#
    .to_string()
}

fn refs_response_mixed() -> String {
    r#"{
        "data": [
            {
                "citedPaper": {
                    "title": "Paper with arXiv",
                    "authors": [{"name": "Author A"}],
                    "year": 2000,
                    "externalIds": {"ArXiv": "cond-mat/0007235"}
                }
            },
            {
                "citedPaper": {
                    "title": "Paper without arXiv",
                    "authors": [{"name": "Author B"}],
                    "year": 1999,
                    "externalIds": {"DOI": "10.1103/abc"}
                }
            },
            {
                "citedPaper": {
                    "title": "Another arXiv paper",
                    "authors": [{"name": "Author C"}],
                    "year": 1999,
                    "externalIds": {"ArXiv": "cond-mat/9910332"}
                }
            }
        ],
        "next": null
    }"#
    .to_string()
}

fn source_with(mock_uri: String) -> SemanticScholarSource {
    SemanticScholarSource::new(reqwest::Client::new())
        .with_base_url(mock_uri)
        .with_rate_limit(Duration::ZERO)
        .with_backoff_base(Duration::from_millis(10))
}

// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_paper_happy_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(paper_response()))
        .mount(&server)
        .await;

    let source = source_with(server.uri());
    let paper = source.fetch_paper("cond-mat/0010317").await.unwrap();

    assert_eq!(paper.title, "Epidemic spreading in scale-free networks");
    assert_eq!(paper.id, "cond-mat/0010317");
    assert_eq!(paper.published, "2001-04-02");
    assert_eq!(paper.citation_count, Some(4200));
}

#[tokio::test]
async fn test_fetch_references_extracts_arxiv_ids() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(refs_response_mixed()))
        .mount(&server)
        .await;

    let mut source = source_with(server.uri());
    let mut paper = Paper { id: "cond-mat/0010317".to_string(), ..Default::default() };
    source.fetch_references(&mut paper).await.unwrap();

    assert_eq!(paper.references.len(), 3);
    let arxiv_ids = paper.get_arxiv_references_ids();
    assert_eq!(arxiv_ids.len(), 2);
    assert!(arxiv_ids.contains(&"cond-mat/0007235".to_string()));
    assert!(arxiv_ids.contains(&"cond-mat/9910332".to_string()));
}

#[tokio::test]
async fn test_fetch_paper_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let source = source_with(server.uri());
    let result = source.fetch_paper("cond-mat/9999999").await;
    assert!(matches!(result, Err(ResynError::PaperNotFound(_))));
}

#[tokio::test]
async fn test_fetch_paper_429_then_success() {
    let server = MockServer::start().await;
    // First two requests return 429, third returns 200.
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(paper_response()))
        .mount(&server)
        .await;

    let source = source_with(server.uri());
    let result = source.fetch_paper("cond-mat/0010317").await;
    assert!(result.is_ok(), "should succeed after retrying: {result:?}");
}

#[tokio::test]
async fn test_fetch_paper_429_persistent_fails() {
    let server = MockServer::start().await;
    // Always returns 429 — exhausts all 3 retries.
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let source = source_with(server.uri());
    let result = source.fetch_paper("cond-mat/0010317").await;
    match result {
        Err(ResynError::SemanticScholarApi(msg)) => {
            assert!(msg.contains("rate limited after 3 retries"), "msg: {msg}");
        }
        other => panic!("expected SemanticScholarApi error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_fetch_paper_malformed_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&server)
        .await;

    let source = source_with(server.uri());
    let result = source.fetch_paper("cond-mat/0010317").await;
    match result {
        Err(ResynError::SemanticScholarApi(msg)) => {
            assert!(msg.contains("failed to parse response"), "msg: {msg}");
        }
        other => panic!("expected SemanticScholarApi parse error, got {other:?}"),
    }
}
