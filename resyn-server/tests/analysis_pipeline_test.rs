//! Analysis pipeline integration test stubs.
//! Wave 0: Define test contracts. Implementation in Plan 04.

#[tokio::test]
#[ignore = "Wave 0 stub -- implemented in Plan 04"]
async fn test_analysis_pipeline_noop_nlp_only() {
    // LLM-01: NLP-only mode when no LLM provider configured (D-07)
    todo!("Verify pipeline with llm_provider=None produces TF-IDF results only")
}

#[tokio::test]
#[ignore = "Wave 0 stub -- implemented in Plan 04"]
async fn test_analysis_pipeline_noop_provider() {
    // LLM-01: Pipeline completes with noop LLM provider
    todo!("Verify run_analysis_pipeline with noop provider returns Ok and produces annotations")
}

#[tokio::test]
#[ignore = "Wave 0 stub -- implemented in Plan 04"]
async fn test_analysis_pipeline_wiremock_ollama() {
    // LLM-01, LLM-02, LLM-03, LLM-04: Full pipeline with wiremock Ollama mock (D-11)
    todo!("Verify pipeline with mocked Ollama produces annotations, gaps, open problems, methods")
}

#[tokio::test]
#[ignore = "Wave 0 stub -- implemented in Plan 04"]
async fn test_analysis_pipeline_caching() {
    // LLM-01: Second pipeline run uses cache, no duplicate analysis (D-02)
    todo!("Verify second run returns Ok and analysis count is still 3, not 6")
}

#[tokio::test]
#[ignore = "Wave 0 stub -- implemented in Plan 04"]
async fn test_start_analysis_http() {
    // LLM-01: StartAnalysis server function responds to HTTP POST (D-10)
    todo!("Verify POST /api/StartAnalysis returns 200 via in-process axum test")
}

#[cfg(feature = "ollama-test")]
#[tokio::test]
#[ignore = "Wave 0 stub -- implemented in Plan 04"]
async fn test_analysis_pipeline_real_ollama() {
    // LLM-01: Optional integration test hitting real Ollama instance (D-11)
    todo!("Verify pipeline completes with real Ollama at OLLAMA_URL")
}
