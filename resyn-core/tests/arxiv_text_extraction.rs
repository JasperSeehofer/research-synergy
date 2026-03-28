//! Integration tests for arXiv HTML text extraction (Phase 18).
//!
//! Uses a real HTML fixture from arxiv.org/html/2503.18887 (per D-05)
//! augmented with two synthetic entries that isolate the Plan 18-01 fix:
//! plain-text arXiv ID extraction (new-format and old-format) and DOI extraction.
//!
//! The fixture has:
//!   - 64 real references from the live page (21 with arxiv.org/abs/ hrefs)
//!   - 1 synthetic entry with a plain-text new-format arXiv ID (arXiv:2501.99999)
//!   - 1 synthetic entry with a plain-text old-format ID (hep-ph/0601234) and plain-text DOI

use resyn_core::data_aggregation::arxiv_utils::aggregate_references_for_arxiv_paper;
use resyn_core::data_aggregation::html_parser::ArxivHTMLDownloader;
use resyn_core::datamodels::paper::Paper;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_HTML: &str = include_str!("fixtures/arxiv_2503_18887_biblio.html");

/// Helper: set up a wiremock server that serves the HTML fixture at the expected path.
async fn setup_mock_server() -> (MockServer, Paper) {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/html/2503.18887"))
        .respond_with(ResponseTemplate::new(200).set_body_string(FIXTURE_HTML))
        .mount(&mock_server)
        .await;

    let mut paper = Paper::new();
    paper.id = "2503.18887".to_string();
    // pdf_url drives convert_pdf_url_to_html_url: /pdf/ID -> /html/ID
    paper.pdf_url = format!("{}/pdf/2503.18887", mock_server.uri());

    (mock_server, paper)
}

/// Verify that aggregate_references_for_arxiv_paper extracts arXiv IDs from
/// the real HTML fixture, covering all three extraction paths:
///
/// 1. `<a href="https://arxiv.org/abs/NNNN.NNNNN">` tags (existing behavior)
/// 2. Plain-text new-format IDs: `arXiv:2501.99999` (no href — Plan 18-01 fix)
/// 3. Plain-text old-format IDs: `hep-ph/0601234` (no href — Plan 18-01 fix)
///
/// Also verifies DOI extraction from plain-text: `10.1103/PhysRevD.99.012345`.
#[tokio::test]
async fn test_arxiv_text_extraction_from_real_html() {
    let (mock_server, mut paper) = setup_mock_server().await;

    let client = reqwest::Client::new();
    let mut downloader = ArxivHTMLDownloader::new(client).with_rate_limit(Duration::from_millis(0));

    aggregate_references_for_arxiv_paper(&mut paper, &mut downloader)
        .await
        .expect("should parse real HTML fixture successfully");

    drop(mock_server);

    // The fixture has 64 real + 2 synthetic = 66 ltx_bibblock elements
    assert!(
        paper.references.len() >= 60,
        "Expected at least 60 references from real HTML fixture (66 total), got {}",
        paper.references.len()
    );

    let arxiv_ids = paper.get_arxiv_references_ids();

    // Real page: 21 references have arxiv.org/abs/ hrefs.
    // Synthetic: 2501.99999 (new-format plain text) and hep-ph/0601234 (old-format plain text).
    // Total expected: >= 21 (real) + 2 (synthetic) = 23.
    assert!(
        arxiv_ids.len() >= 23,
        "Expected at least 23 arXiv reference IDs (21 real hrefs + 2 synthetic), \
         got {}. ids (first 10): {:?}",
        arxiv_ids.len(),
        &arxiv_ids[..arxiv_ids.len().min(10)]
    );

    // Verify the new-format plain-text ID (synthetic entry bib.bib65, no href).
    // This is the core test for Plan 18-01: arXiv:2501.99999 has NO <a> tag.
    // get_arxiv_id() returns the last path segment, so "2501.99999" is expected.
    assert!(
        arxiv_ids.iter().any(|id| id == "2501.99999"),
        "Expected to extract arXiv ID 2501.99999 from plain text (no href). \
         This tests the new-format regex path of Plan 18-01. got: {:?}",
        &arxiv_ids[..arxiv_ids.len().min(10)]
    );

    // Verify the old-format plain-text ID (synthetic entry bib.bib66, no href).
    // get_arxiv_id() splits the URL by '/' and returns the last segment.
    // For https://arxiv.org/abs/hep-ph/0601234, the last segment is "0601234".
    // We also verify via arxiv_eprint that the full ID "hep-ph/0601234" was captured.
    let old_format_eprint = paper
        .references
        .iter()
        .find(|r| r.arxiv_eprint.as_deref() == Some("hep-ph/0601234"));
    assert!(
        old_format_eprint.is_some(),
        "Expected to find a reference with arxiv_eprint = 'hep-ph/0601234' \
         (old-format plain-text extraction, synthetic entry bib.bib66). \
         This tests the old-format regex path of Plan 18-01."
    );

    // Verify arxiv_eprint is populated for at least the 2 synthetic plain-text entries.
    let eprint_count = paper
        .references
        .iter()
        .filter(|r| r.arxiv_eprint.is_some())
        .count();
    assert!(
        eprint_count >= 2,
        "Expected at least 2 references with arxiv_eprint set \
         (both synthetic plain-text entries), got {eprint_count}. \
         This indicates Plan 18-01 text extraction is not working."
    );

    // Verify DOI extraction from plain text (synthetic entry bib.bib66).
    // The reference contains `10.1103/PhysRevD.99.012345` as plain text.
    let doi_count = paper.references.iter().filter(|r| r.doi.is_some()).count();
    assert!(
        doi_count >= 1,
        "Expected at least 1 reference with a DOI extracted from plain text, \
         got {doi_count}. Synthetic entry bib.bib66 has 10.1103/PhysRevD.99.012345."
    );
}

/// Compare arXiv-extracted edge count against total reference count.
///
/// Requires extracting arXiv IDs from >= 70% of the arXiv-annotated references
/// in the fixture. The fixture has 23 arXiv-annotated entries:
/// 21 real (via href) + 2 synthetic (via plain text). All 23 should be extracted.
///
/// Per D-04: verify "comparable edge density" via automated test.
#[tokio::test]
async fn test_arxiv_edge_density_comparable() {
    let (mock_server, mut paper) = setup_mock_server().await;

    let client = reqwest::Client::new();
    let mut downloader = ArxivHTMLDownloader::new(client).with_rate_limit(Duration::from_millis(0));

    aggregate_references_for_arxiv_paper(&mut paper, &mut downloader)
        .await
        .expect("should parse real HTML fixture successfully");

    drop(mock_server);

    let arxiv_edge_count = paper.get_arxiv_references_ids().len();
    let total_references = paper.references.len();

    // Require at least 23 arXiv edges (21 real + 2 synthetic).
    // After Plan 18-01, both href-based and plain-text IDs are captured.
    assert!(
        arxiv_edge_count >= 23,
        "Expected at least 23 arXiv reference IDs from {total_references} total references, \
         got {arxiv_edge_count}. The text extraction fix (Plan 18-01) should capture \
         both the 21 real href-linked IDs and 2 synthetic plain-text IDs."
    );

    eprintln!(
        "Edge density: {arxiv_edge_count}/{total_references} references \
         have arXiv IDs ({:.0}%)",
        (arxiv_edge_count as f64 / total_references as f64) * 100.0
    );
}
