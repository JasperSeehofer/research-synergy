use leptos::prelude::*;

/// Check whether an LLM provider is configured via the `RESYN_LLM_PROVIDER` environment variable.
///
/// Returns `true` if the variable is set (any non-empty value), `false` otherwise.
/// Used by the frontend to show the LLM warning banner (D-07).
#[server(CheckLlmConfigured, "/api")]
pub async fn check_llm_configured() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        Ok(std::env::var("RESYN_LLM_PROVIDER").is_ok())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Trigger the analysis pipeline in the background.
///
/// Returns immediately with "Analysis started". Progress events are broadcast
/// via the SSE `/progress` endpoint with `event_type` values:
/// - `"analysis_extracting"` — ar5iv text extraction stage
/// - `"analysis_nlp"` — TF-IDF corpus analysis stage
/// - `"analysis_llm"` — LLM paper annotation stage (only if RESYN_LLM_PROVIDER set)
/// - `"analysis_gaps"` — contradiction + ABC-bridge detection stage
/// - `"analysis_complete"` — pipeline finished successfully
/// - `"analysis_error"` — pipeline failed
///
/// When `RESYN_LLM_PROVIDER` is unset, the pipeline runs NLP-only and still
/// completes successfully (per D-07).
#[server(StartAnalysis, "/api")]
pub async fn start_analysis() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::datamodels::progress::ProgressEvent;
        use std::sync::Arc;
        use tokio::sync::broadcast;
        use tracing::{error, info};

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let tx = use_context::<broadcast::Sender<ProgressEvent>>()
            .ok_or_else(|| ServerFnError::new("Progress channel not available"))?;

        tokio::spawn(async move {
            let started = std::time::Instant::now();

            // Broadcast an analysis progress event.
            let send_progress =
                |tx: &broadcast::Sender<ProgressEvent>, event_type: &str, stage: &str| {
                    let _ = tx.send(ProgressEvent {
                        event_type: event_type.to_string(),
                        papers_found: 0,
                        papers_pending: 0,
                        papers_failed: 0,
                        current_depth: 0,
                        max_depth: 0,
                        elapsed_secs: started.elapsed().as_secs_f64(),
                        current_paper_id: None,
                        current_paper_title: None,
                        analysis_stage: Some(stage.to_string()),
                    });
                };

            // Read LLM config from environment (per D-06).
            let llm_provider_name = std::env::var("RESYN_LLM_PROVIDER").ok();
            let llm_model = std::env::var("RESYN_LLM_MODEL").ok();

            // --- Stage 1: text extraction ---
            send_progress(&tx, "analysis_extracting", "extracting");
            {
                use resyn_core::data_aggregation::text_extractor::Ar5ivExtractor;
                use resyn_core::database::queries::ExtractionRepository;
                use resyn_core::database::queries::PaperRepository;
                use resyn_core::utils::strip_version_suffix;

                let extraction_repo = ExtractionRepository::new(&db);
                let paper_repo = PaperRepository::new(&db);
                let all_papers = match paper_repo.get_all_papers().await {
                    Ok(p) => p,
                    Err(e) => {
                        error!(error = %e, "Failed to load papers for extraction");
                        send_progress(&tx, "analysis_error", "error");
                        return;
                    }
                };

                let client = resyn_core::utils::create_http_client();
                let mut extractor =
                    Ar5ivExtractor::new(client).with_rate_limit(std::time::Duration::from_secs(3));

                for paper in &all_papers {
                    let stripped_id = strip_version_suffix(&paper.id);
                    if extraction_repo
                        .extraction_exists(&stripped_id)
                        .await
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    let result = extractor.extract(paper).await;
                    if let Err(e) = extraction_repo.upsert_extraction(&result).await {
                        error!(paper_id = paper.id, error = %e, "Failed to store extraction");
                    }
                }
                info!(total = all_papers.len(), "Extraction stage complete");
            }

            // --- Stage 2: NLP / TF-IDF ---
            send_progress(&tx, "analysis_nlp", "nlp");
            {
                use resyn_core::database::queries::{AnalysisRepository, ExtractionRepository};
                use resyn_core::datamodels::analysis::{AnalysisMetadata, PaperAnalysis};
                use resyn_core::nlp::tfidf::{TfIdfEngine, corpus_fingerprint};

                let extraction_repo = ExtractionRepository::new(&db);
                let analysis_repo = AnalysisRepository::new(&db);

                let extractions = match extraction_repo.get_all_extractions().await {
                    Ok(e) => e,
                    Err(e) => {
                        error!(error = %e, "Failed to load extractions for NLP");
                        // NLP failure is non-fatal — continue to LLM stage.
                        vec![]
                    }
                };

                if !extractions.is_empty() {
                    let arxiv_ids: Vec<String> =
                        extractions.iter().map(|e| e.arxiv_id.clone()).collect();
                    let fingerprint = corpus_fingerprint(&arxiv_ids);
                    let paper_count = extractions.len() as u64;

                    let already_done = analysis_repo
                        .get_metadata("corpus_tfidf")
                        .await
                        .ok()
                        .flatten()
                        .map(|m| m.corpus_fingerprint == fingerprint && m.paper_count == paper_count)
                        .unwrap_or(false);

                    if !already_done {
                        let tfidf_results = TfIdfEngine::compute_corpus(&extractions);
                        for (arxiv_id, tfidf_vector) in &tfidf_results {
                            let (top_terms, top_scores) =
                                TfIdfEngine::get_top_n(tfidf_vector, 5);
                            let analysis = PaperAnalysis {
                                arxiv_id: arxiv_id.clone(),
                                tfidf_vector: tfidf_vector.clone(),
                                top_terms,
                                top_scores,
                                analyzed_at: chrono::Utc::now().to_rfc3339(),
                                corpus_fingerprint: fingerprint.clone(),
                            };
                            if let Err(e) = analysis_repo.upsert_analysis(&analysis).await {
                                error!(paper_id = arxiv_id.as_str(), error = %e, "Failed to persist NLP analysis");
                            }
                        }
                        let metadata = AnalysisMetadata {
                            key: "corpus_tfidf".to_string(),
                            paper_count,
                            corpus_fingerprint: fingerprint,
                            last_analyzed: chrono::Utc::now().to_rfc3339(),
                        };
                        if let Err(e) = analysis_repo.upsert_metadata(&metadata).await {
                            error!(error = %e, "Failed to persist NLP metadata");
                        }
                        info!(papers = paper_count, "NLP stage complete");
                    } else {
                        info!("NLP stage skipped — corpus unchanged");
                    }
                }
            }

            // --- Stages 3 & 4: LLM annotation + gap analysis (optional) ---
            if let Some(ref provider_name) = llm_provider_name {
                use resyn_core::llm::claude::ClaudeProvider;
                use resyn_core::llm::noop::NoopProvider;
                use resyn_core::llm::ollama::OllamaProvider;
                use resyn_core::llm::traits::LlmProvider;

                let client = resyn_core::utils::create_http_client();
                let mut provider: Box<dyn LlmProvider> = match provider_name.as_str() {
                    "claude" => match ClaudeProvider::new(client) {
                        Ok(p) => Box::new(if let Some(ref m) = llm_model {
                            p.with_model(m.clone())
                        } else {
                            p
                        }),
                        Err(e) => {
                            error!(error = %e, "Failed to initialize Claude provider");
                            send_progress(&tx, "analysis_error", "error");
                            return;
                        }
                    },
                    "ollama" => {
                        let p = OllamaProvider::new(client);
                        Box::new(if let Some(ref m) = llm_model {
                            p.with_model(m.clone())
                        } else {
                            p
                        })
                    }
                    "noop" => Box::new(NoopProvider),
                    other => {
                        error!(provider = other, "Unknown LLM provider — use: claude, ollama, noop");
                        send_progress(&tx, "analysis_error", "error");
                        return;
                    }
                };

                // Stage 3: LLM annotation
                send_progress(&tx, "analysis_llm", "llm");
                {
                    use resyn_core::database::queries::{LlmAnnotationRepository, PaperRepository};
                    use resyn_core::utils::strip_version_suffix;

                    let paper_repo = PaperRepository::new(&db);
                    let llm_repo = LlmAnnotationRepository::new(&db);

                    let all_papers = match paper_repo.get_all_papers().await {
                        Ok(p) => p,
                        Err(e) => {
                            error!(error = %e, "Failed to load papers for LLM analysis");
                            send_progress(&tx, "analysis_error", "error");
                            return;
                        }
                    };

                    let (mut annotated, mut skipped, mut failed) = (0usize, 0usize, 0usize);
                    for paper in &all_papers {
                        let id = strip_version_suffix(&paper.id);
                        if llm_repo.annotation_exists(&id).await.unwrap_or(false) {
                            skipped += 1;
                            continue;
                        }
                        match provider.annotate_paper(&id, &paper.summary).await {
                            Ok(ann) => {
                                if let Err(e) = llm_repo.upsert_annotation(&ann).await {
                                    error!(paper_id = id.as_str(), error = %e, "Failed to persist annotation");
                                }
                                annotated += 1;
                            }
                            Err(e) => {
                                error!(paper_id = id.as_str(), error = %e, "LLM annotation failed");
                                failed += 1;
                            }
                        }
                    }
                    info!(annotated, skipped, failed, "LLM annotation stage complete");
                }

                // Stage 4: gap analysis
                send_progress(&tx, "analysis_gaps", "gaps");
                {
                    use resyn_core::database::queries::{
                        AnalysisRepository, GapFindingRepository, LlmAnnotationRepository,
                        PaperRepository,
                    };
                    use resyn_core::datamodels::analysis::AnalysisMetadata;
                    use resyn_core::nlp::tfidf::corpus_fingerprint;

                    let analysis_repo = AnalysisRepository::new(&db);
                    let llm_repo = LlmAnnotationRepository::new(&db);
                    let paper_repo = PaperRepository::new(&db);
                    let gap_repo = GapFindingRepository::new(&db);

                    let analyses = analysis_repo.get_all_analyses().await.unwrap_or_default();
                    let annotations = llm_repo.get_all_annotations().await.unwrap_or_default();

                    if !analyses.is_empty() && !annotations.is_empty() {
                        let annotation_ids: Vec<String> =
                            annotations.iter().map(|a| a.arxiv_id.clone()).collect();
                        let fingerprint = corpus_fingerprint(&annotation_ids);

                        let already_done = analysis_repo
                            .get_metadata("gap_analysis")
                            .await
                            .ok()
                            .flatten()
                            .map(|m| m.corpus_fingerprint == fingerprint)
                            .unwrap_or(false);

                        if !already_done {
                            let papers = paper_repo.get_all_papers().await.unwrap_or_default();
                            let graph =
                                resyn_core::data_processing::graph_creation::create_graph_from_papers(
                                    &papers,
                                );
                            let contradictions =
                                resyn_core::gap_analysis::contradiction::find_contradictions(
                                    &analyses,
                                    &annotations,
                                    provider.as_mut(),
                                )
                                .await;
                            let abc_bridges =
                                resyn_core::gap_analysis::abc_bridge::find_abc_bridges(
                                    &analyses,
                                    &annotations,
                                    &graph,
                                    provider.as_mut(),
                                    false,
                                )
                                .await;

                            for finding in contradictions.iter().chain(abc_bridges.iter()) {
                                if let Err(e) = gap_repo.insert_gap_finding(finding).await {
                                    error!(error = %e, "Failed to persist gap finding");
                                }
                            }

                            let metadata = AnalysisMetadata {
                                key: "gap_analysis".to_string(),
                                paper_count: annotation_ids.len() as u64,
                                corpus_fingerprint: fingerprint,
                                last_analyzed: chrono::Utc::now().to_rfc3339(),
                            };
                            if let Err(e) = analysis_repo.upsert_metadata(&metadata).await {
                                error!(error = %e, "Failed to persist gap metadata");
                            }
                            info!("Gap analysis stage complete");
                        } else {
                            info!("Gap analysis skipped — corpus unchanged");
                        }
                    }
                }
            }

            send_progress(&tx, "analysis_complete", "complete");
            info!(
                elapsed_secs = started.elapsed().as_secs_f64(),
                "Analysis pipeline completed successfully"
            );
        });

        Ok("Analysis started".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
