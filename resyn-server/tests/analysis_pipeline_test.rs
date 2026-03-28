//! Analysis pipeline integration tests.
//! Plan 04: Full implementation of Wave 0 stubs.

use resyn_core::database::client::connect_memory;
use resyn_core::database::queries::{
    AnalysisRepository, GapFindingRepository, LlmAnnotationRepository, PaperRepository,
};
use resyn_core::datamodels::paper::Paper;
use resyn_server::commands::analyze::{AnalyzeArgs, run_analysis_pipeline};

// ── Test fixtures ──────────────────────────────────────────────────────────────

/// Valid LLM annotation JSON in the format `LlmAnnotationRaw` expects.
const VALID_ANNOTATION_JSON: &str = r#"{
    "paper_type": "computational",
    "methods": [{"name": "Monte Carlo simulation", "category": "computational"},
                {"name": "Bayesian inference", "category": "statistical"}],
    "findings": [{"text": "Improved sampling efficiency by 40%", "strength": "strong_evidence"}],
    "open_problems": ["Scaling to larger lattice volumes remains challenging"]
}"#;

fn make_ollama_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "message": {"role": "assistant", "content": content},
        "done": true
    })
}

/// Seed an in-memory DB with 3 test papers and return the `Db` handle.
async fn seed_test_db() -> resyn_core::database::client::Db {
    let db = connect_memory()
        .await
        .expect("Failed to connect in-memory DB");
    let repo = PaperRepository::new(&db);

    let papers = vec![
        Paper {
            id: "2301.00001".to_string(),
            title: "Monte Carlo methods for lattice QCD simulations".to_string(),
            summary: "We present novel Monte Carlo simulation techniques for lattice quantum chromodynamics. Our method uses Bayesian inference to improve sampling efficiency in the Markov chain.".to_string(),
            ..Default::default()
        },
        Paper {
            id: "2301.00002".to_string(),
            title: "Bayesian inference in particle physics measurements".to_string(),
            summary: "This paper applies Bayesian statistical methods to particle physics detector measurements. We use neural network classifiers trained on Monte Carlo simulated events.".to_string(),
            ..Default::default()
        },
        Paper {
            id: "2301.00003".to_string(),
            title: "Neural network approaches to jet classification".to_string(),
            summary: "Deep neural networks are applied to classify hadronic jets in proton-proton collisions. The method combines convolutional and recurrent architectures for improved discrimination.".to_string(),
            ..Default::default()
        },
    ];

    for paper in &papers {
        repo.upsert_paper(paper)
            .await
            .expect("Failed to upsert test paper");
    }

    db
}

fn make_noop_args() -> AnalyzeArgs {
    AnalyzeArgs {
        db: String::new(),
        llm_provider: None,
        llm_model: None,
        force: false,
        full_corpus: false,
        verbose: false,
    }
}

fn make_noop_llm_args() -> AnalyzeArgs {
    AnalyzeArgs {
        db: String::new(),
        llm_provider: Some("noop".to_string()),
        llm_model: None,
        force: false,
        full_corpus: false,
        verbose: false,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

/// LLM-01 / D-07: NLP-only mode produces TF-IDF results and no LLM annotations.
#[tokio::test]
async fn test_analysis_pipeline_noop_nlp_only() {
    let db = seed_test_db().await;
    let args = make_noop_args();

    let result = run_analysis_pipeline(&db, args, 0, true).await;
    assert!(
        result.is_ok(),
        "NLP-only pipeline should succeed: {:?}",
        result.err()
    );

    let analysis_repo = AnalysisRepository::new(&db);
    let analyses = analysis_repo
        .get_all_analyses()
        .await
        .expect("Failed to query analyses");
    assert_eq!(
        analyses.len(),
        3,
        "Expect 3 TF-IDF analyses, one per paper (got {})",
        analyses.len()
    );

    let annotation_repo = LlmAnnotationRepository::new(&db);
    let annotations = annotation_repo
        .get_all_annotations()
        .await
        .expect("Failed to query annotations");
    assert_eq!(
        annotations.len(),
        0,
        "NLP-only mode should produce 0 LLM annotations (got {})",
        annotations.len()
    );
}

/// LLM-01: Pipeline completes with noop LLM provider.
#[tokio::test]
async fn test_analysis_pipeline_noop_provider() {
    let db = seed_test_db().await;
    let args = make_noop_llm_args();

    let result = run_analysis_pipeline(&db, args, 0, true).await;
    assert!(
        result.is_ok(),
        "Noop-provider pipeline should succeed: {:?}",
        result.err()
    );

    let analysis_repo = AnalysisRepository::new(&db);
    let analyses = analysis_repo
        .get_all_analyses()
        .await
        .expect("Failed to query analyses");
    assert_eq!(
        analyses.len(),
        3,
        "Expect 3 TF-IDF analyses (got {})",
        analyses.len()
    );

    let annotation_repo = LlmAnnotationRepository::new(&db);
    let annotations = annotation_repo
        .get_all_annotations()
        .await
        .expect("Failed to query annotations");
    assert_eq!(
        annotations.len(),
        3,
        "Noop provider should annotate all 3 papers (got {})",
        annotations.len()
    );

    // Gap findings may be 0 (noop verify_gap returns "NO"), which is acceptable.
    let gap_repo = GapFindingRepository::new(&db);
    let _gaps = gap_repo
        .get_all_gap_findings()
        .await
        .expect("Failed to query gap findings");
}

/// LLM-01, LLM-02, LLM-03, LLM-04 / D-11: Full pipeline with wiremock Ollama mock.
#[tokio::test]
async fn test_analysis_pipeline_wiremock_ollama() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let db = seed_test_db().await;

    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(make_ollama_response(VALID_ANNOTATION_JSON)),
        )
        .mount(&mock_server)
        .await;

    // Point OllamaProvider at the mock server.
    // Safety: test-only env mutation; tests are assumed to run in a process where
    // concurrent env mutation is controlled (single test or serial execution).
    unsafe {
        std::env::set_var("OLLAMA_URL", mock_server.uri());
    }

    let args = AnalyzeArgs {
        db: String::new(),
        llm_provider: Some("ollama".to_string()),
        llm_model: Some("llama3.2".to_string()),
        force: false,
        full_corpus: false,
        verbose: false,
    };

    let result = run_analysis_pipeline(&db, args, 0, true).await;
    assert!(
        result.is_ok(),
        "Wiremock Ollama pipeline should succeed: {:?}",
        result.err()
    );

    let annotation_repo = LlmAnnotationRepository::new(&db);
    let annotations = annotation_repo
        .get_all_annotations()
        .await
        .expect("Failed to query annotations");
    assert_eq!(
        annotations.len(),
        3,
        "Wiremock Ollama should produce 3 annotations (got {})",
        annotations.len()
    );

    // LLM-03: open problems should be non-empty (each annotation has 1 open problem).
    let open_problems =
        resyn_core::analysis::aggregation::aggregate_open_problems(&annotations);
    assert!(
        !open_problems.is_empty(),
        "aggregate_open_problems should return non-empty results"
    );

    // LLM-04: method matrix should have non-empty categories.
    let method_matrix = resyn_core::analysis::aggregation::build_method_matrix(&annotations);
    assert!(
        !method_matrix.categories.is_empty(),
        "build_method_matrix should return non-empty categories"
    );

    // LLM-02: gap findings (may be empty due to noop verify_gap on contradictions, but should not error).
    let gap_repo = GapFindingRepository::new(&db);
    let _gap_findings = gap_repo
        .get_all_gap_findings()
        .await
        .expect("Failed to query gap findings");
}

/// D-02: Second pipeline run uses cached results, no duplicate analyses.
#[tokio::test]
async fn test_analysis_pipeline_caching() {
    let db = seed_test_db().await;

    // First run.
    let result1 = run_analysis_pipeline(&db, make_noop_args(), 0, true).await;
    assert!(
        result1.is_ok(),
        "First pipeline run should succeed: {:?}",
        result1.err()
    );

    // Second run with the same args — should use cached corpus fingerprint.
    let result2 = run_analysis_pipeline(&db, make_noop_args(), 0, true).await;
    assert!(
        result2.is_ok(),
        "Second pipeline run (cached) should succeed: {:?}",
        result2.err()
    );

    let analysis_repo = AnalysisRepository::new(&db);
    let analyses = analysis_repo
        .get_all_analyses()
        .await
        .expect("Failed to query analyses");
    assert_eq!(
        analyses.len(),
        3,
        "After two runs, analysis count should still be 3 (not 6): got {}",
        analyses.len()
    );
}

/// D-10: POST /api/StartAnalysis returns 200 via in-process axum test.
#[tokio::test]
async fn test_start_analysis_http() {
    use axum::body::Body;
    use http::Request;
    use leptos::prelude::provide_context;
    use leptos_axum::handle_server_fns_with_context;
    use resyn_app::server_fns::analysis::StartAnalysis;
    use resyn_core::database::client::connect_memory;
    use resyn_core::datamodels::progress::ProgressEvent;
    use server_fn::axum::register_explicit;
    use server_fn::ServerFn;
    use std::sync::Arc;
    use tokio::sync::broadcast;
    use tower::ServiceExt;

    // Register StartAnalysis server function.
    register_explicit::<StartAnalysis>();

    // Leptos 0.8 generates hash-suffixed paths (e.g. /api/start_analysis<hash>).
    // Use StartAnalysis::PATH to get the exact registered path at compile time.
    let fn_path = StartAnalysis::PATH;

    // Set up in-memory DB and broadcast channel.
    let db = connect_memory()
        .await
        .expect("Failed to connect in-memory DB");
    let db = Arc::new(db);
    let (progress_tx, _) = broadcast::channel::<ProgressEvent>(256);

    let db_for_fns = db.clone();
    let tx_for_fns = progress_tx.clone();

    // Build minimal axum router mirroring serve.rs.
    // Route pattern must match fn_path (e.g. /api/start_analysis<hash>).
    let app = axum::Router::new().route(
        "/api/{*fn_name}",
        axum::routing::post({
            let db = db_for_fns.clone();
            let tx = tx_for_fns.clone();
            move |req| {
                handle_server_fns_with_context(
                    move || {
                        provide_context(db.clone());
                        provide_context(tx.clone());
                    },
                    req,
                )
            }
        }),
    );

    // Build the POST request using the exact registered path.
    // Leptos 0.8 server functions use PostUrl encoding by default:
    // content-type: application/x-www-form-urlencoded, empty body for no-arg fns.
    let request = Request::builder()
        .method("POST")
        .uri(fn_path)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Accept 200 (success) or 202 (accepted/started), as the server function
    // returns immediately with "Analysis started" and spawns a background task.
    let status = response.status();
    assert!(
        status.is_success(),
        "POST /api/StartAnalysis should return 2xx, got {status}"
    );
}

/// D-11: Feature-gated real Ollama integration test.
/// Requires a running Ollama instance at OLLAMA_URL (default http://localhost:11434).
/// Run with: cargo test --package resyn-server --features ollama-test
#[cfg(feature = "ollama-test")]
#[tokio::test]
async fn test_analysis_pipeline_real_ollama() {
    let db = seed_test_db().await;
    let args = AnalyzeArgs {
        db: String::new(),
        llm_provider: Some("ollama".to_string()),
        llm_model: Some("llama3.2:1b".to_string()),
        force: false,
        full_corpus: false,
        verbose: true,
    };
    let result = run_analysis_pipeline(&db, args, 0, true).await;
    assert!(
        result.is_ok(),
        "Pipeline with real Ollama failed: {:?}",
        result.err()
    );

    let annotation_repo = LlmAnnotationRepository::new(&db);
    let annotations = annotation_repo
        .get_all_annotations()
        .await
        .expect("Failed to query annotations");
    assert!(
        !annotations.is_empty(),
        "Real Ollama should produce annotations"
    );
}
