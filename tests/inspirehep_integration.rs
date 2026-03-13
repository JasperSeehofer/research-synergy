use research_synergy::data_aggregation::inspirehep_api::InspireHepClient;
use research_synergy::data_aggregation::traits::PaperSource;
use research_synergy::datamodels::paper::DataSource;
use std::time::Duration;
use wiremock::matchers::{method, query_param_contains};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_literature_response() -> String {
    r#"{
        "hits": {
            "hits": [{
                "id": 1234567,
                "metadata": {
                    "titles": [{"title": "Test Paper from InspireHEP"}],
                    "authors": [
                        {"full_name": "Doe, John"},
                        {"full_name": "Smith, Jane"}
                    ],
                    "abstracts": [{"value": "An abstract about physics."}],
                    "arxiv_eprints": [{"value": "2301.12345"}],
                    "dois": [{"value": "10.1234/test.2023"}],
                    "citation_count": 42,
                    "references": [
                        {
                            "reference": {
                                "title": {"title": "Referenced Paper One"},
                                "authors": [{"full_name": "Ref, Author"}],
                                "arxiv_eprint": "2301.11111",
                                "dois": ["10.1234/ref1.2023"]
                            },
                            "record": {"$ref": "https://inspirehep.net/api/literature/9999999"},
                            "label": "1"
                        },
                        {
                            "reference": {
                                "title": {"title": "Non-arXiv Paper"},
                                "authors": [{"full_name": "Other, Person"}],
                                "dois": ["10.1234/ref2.2023"]
                            },
                            "record": {"$ref": "https://inspirehep.net/api/literature/8888888"},
                            "label": "2"
                        }
                    ]
                }
            }]
        }
    }"#
    .to_string()
}

#[tokio::test]
async fn test_fetch_paper_from_inspirehep() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param_contains("q", "arxiv:2301.12345"))
        .respond_with(ResponseTemplate::new(200).set_body_string(sample_literature_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let inspire = InspireHepClient::new(client)
        .with_base_url(mock_server.uri())
        .with_rate_limit(Duration::from_millis(0));

    let paper = inspire.fetch_paper("2301.12345").await.unwrap();

    assert_eq!(paper.title, "Test Paper from InspireHEP");
    assert_eq!(paper.authors, vec!["Doe, John", "Smith, Jane"]);
    assert_eq!(paper.id, "2301.12345");
    assert_eq!(paper.doi, Some("10.1234/test.2023".to_string()));
    assert_eq!(paper.citation_count, Some(42));
    assert_eq!(paper.source, DataSource::InspireHep);
}

#[tokio::test]
async fn test_fetch_references_from_inspirehep() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param_contains("q", "arxiv:2301.12345"))
        .respond_with(ResponseTemplate::new(200).set_body_string(sample_literature_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let mut inspire = InspireHepClient::new(client)
        .with_base_url(mock_server.uri())
        .with_rate_limit(Duration::from_millis(0));

    let mut paper = research_synergy::datamodels::paper::Paper {
        id: "2301.12345".to_string(),
        ..Default::default()
    };

    inspire.fetch_references(&mut paper).await.unwrap();

    assert_eq!(paper.references.len(), 2);
    assert_eq!(paper.references[0].title, "Referenced Paper One");
    assert_eq!(
        paper.references[0].arxiv_eprint,
        Some("2301.11111".to_string())
    );
    assert_eq!(paper.references[0].label, Some("1".to_string()));

    // First ref has arXiv link, second doesn't
    assert_eq!(paper.references[0].links.len(), 1);
    assert_eq!(paper.references[1].links.len(), 0);

    // get_arxiv_references_ids should only return the one with arXiv link
    let arxiv_ids = paper.get_arxiv_references_ids();
    assert_eq!(arxiv_ids.len(), 1);
    assert_eq!(arxiv_ids[0], "2301.11111");
}

#[tokio::test]
async fn test_inspirehep_paper_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param_contains("q", "arxiv:9999.99999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let inspire = InspireHepClient::new(client)
        .with_base_url(mock_server.uri())
        .with_rate_limit(Duration::from_millis(0));

    let result = inspire.fetch_paper("9999.99999").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_inspirehep_malformed_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param_contains("q", "arxiv:2301.12345"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let inspire = InspireHepClient::new(client)
        .with_base_url(mock_server.uri())
        .with_rate_limit(Duration::from_millis(0));

    let result = inspire.fetch_paper("2301.12345").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_inspirehep_empty_hits() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(query_param_contains("q", "arxiv:2301.00000"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"hits": {"hits": []}}"#))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let inspire = InspireHepClient::new(client)
        .with_base_url(mock_server.uri())
        .with_rate_limit(Duration::from_millis(0));

    let result = inspire.fetch_paper("2301.00000").await;
    assert!(result.is_err()); // PaperNotFound
}
