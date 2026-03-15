use chrono::Utc;
use clap::Args;
use resyn_core::database::client::Db;
use resyn_core::database::queries::{
    AnalysisRepository, GapFindingRepository, LlmAnnotationRepository,
};
use resyn_core::datamodels::analysis::{AnalysisMetadata, PaperAnalysis};
use resyn_core::llm::claude::ClaudeProvider;
use resyn_core::llm::noop::NoopProvider;
use resyn_core::llm::ollama::OllamaProvider;
use resyn_core::llm::traits::LlmProvider;
use tracing::{error, info, warn};

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Database connection string (e.g. "surrealkv://./data")
    #[arg(long, default_value = "surrealkv://./data")]
    pub db: String,

    /// LLM provider for semantic extraction: claude, ollama, noop
    #[arg(long)]
    pub llm_provider: Option<String>,

    /// LLM model override (e.g. claude-sonnet-4-20250514, llama3.2)
    #[arg(long)]
    pub llm_model: Option<String>,

    /// Re-analyze already-analyzed papers
    #[arg(long, default_value_t = false)]
    pub force: bool,

    /// Expand ABC-bridge scope to all papers in SurrealDB
    #[arg(long, default_value_t = false)]
    pub full_corpus: bool,

    /// Show full justifications in gap output table (default: truncated at 60 chars)
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

pub async fn run(args: AnalyzeArgs) -> anyhow::Result<()> {
    let db = match resyn_core::database::client::connect(&args.db).await {
        Ok(db) => {
            info!(endpoint = args.db.as_str(), "Connected to database");
            db
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to database");
            std::process::exit(1);
        }
    };

    // Default rate limit for analyze-only mode (no crawl happening)
    let rate_limit_secs = 3u64;
    let skip_fulltext = false;

    run_analysis_pipeline(&db, args, rate_limit_secs, skip_fulltext).await
}

/// Shared analysis pipeline used by both `analyze run()` and `crawl run()` when `--analyze` is set.
pub async fn run_analysis_pipeline(
    db: &Db,
    args: AnalyzeArgs,
    rate_limit_secs: u64,
    skip_fulltext: bool,
) -> anyhow::Result<()> {
    run_extraction(db, rate_limit_secs, skip_fulltext, args.force).await;
    run_nlp_analysis(db).await;

    if let Some(ref provider_name) = args.llm_provider {
        let client = resyn_core::utils::create_http_client();
        let mut provider: Box<dyn LlmProvider> = match provider_name.as_str() {
            "claude" => {
                let p = ClaudeProvider::new(client).unwrap_or_else(|e| {
                    error!(error = %e, "Failed to initialize Claude provider");
                    std::process::exit(1);
                });
                Box::new(if let Some(ref m) = args.llm_model {
                    p.with_model(m.clone())
                } else {
                    p
                })
            }
            "ollama" => {
                let p = OllamaProvider::new(client);
                Box::new(if let Some(ref m) = args.llm_model {
                    p.with_model(m.clone())
                } else {
                    p
                })
            }
            "noop" => Box::new(NoopProvider),
            other => {
                error!(
                    provider = other,
                    "Unknown LLM provider. Use: claude, ollama, noop"
                );
                std::process::exit(1);
            }
        };
        run_llm_analysis(db, provider.as_mut()).await;
        run_gap_analysis(db, provider.as_mut(), args.full_corpus, args.verbose).await;
    }

    Ok(())
}

async fn run_extraction(db: &Db, rate_limit_secs: u64, skip_fulltext: bool, force: bool) {
    let extraction_repo = resyn_core::database::queries::ExtractionRepository::new(db);
    let paper_repo = resyn_core::database::queries::PaperRepository::new(db);
    let all_papers = paper_repo.get_all_papers().await.unwrap_or_else(|e| {
        error!(error = %e, "Failed to load papers for analysis");
        std::process::exit(1);
    });

    let client = resyn_core::utils::create_http_client();
    let mut extractor =
        resyn_core::data_aggregation::text_extractor::Ar5ivExtractor::new(client)
            .with_rate_limit(std::time::Duration::from_secs(rate_limit_secs));

    let mut abstract_only_count: usize = 0;
    let mut skipped_count: usize = 0;
    for paper in &all_papers {
        let stripped_id = resyn_core::utils::strip_version_suffix(&paper.id);
        if !force
            && extraction_repo
                .extraction_exists(&stripped_id)
                .await
                .unwrap_or(false)
        {
            skipped_count += 1;
            continue;
        }
        let result = if skip_fulltext {
            resyn_core::datamodels::extraction::TextExtractionResult::from_abstract(paper)
        } else {
            extractor.extract(paper).await
        };
        if result.is_partial {
            abstract_only_count += 1;
        }
        if let Err(e) = extraction_repo.upsert_extraction(&result).await {
            error!(paper_id = paper.id, error = %e, "Failed to store extraction");
        }
    }

    let analyzed = all_papers.len() - skipped_count;
    info!(
        abstract_only = abstract_only_count,
        analyzed = analyzed,
        skipped = skipped_count,
        total = all_papers.len(),
        "{}/{} papers used abstract-only extraction ({} skipped, already cached)",
        abstract_only_count,
        analyzed,
        skipped_count
    );
}

async fn run_nlp_analysis(db: &Db) {
    let extraction_repo = resyn_core::database::queries::ExtractionRepository::new(db);
    let analysis_repo = AnalysisRepository::new(db);

    let extractions = match extraction_repo.get_all_extractions().await {
        Ok(e) => e,
        Err(err) => {
            error!(error = %err, "Failed to load extractions for NLP analysis");
            return;
        }
    };

    if extractions.is_empty() {
        info!("No extractions found, skipping NLP analysis");
        return;
    }

    let arxiv_ids: Vec<String> = extractions.iter().map(|e| e.arxiv_id.clone()).collect();
    let fingerprint = resyn_core::nlp::tfidf::corpus_fingerprint(&arxiv_ids);
    let paper_count = extractions.len() as u64;

    if let Ok(Some(existing_meta)) = analysis_repo.get_metadata("corpus_tfidf").await
        && existing_meta.corpus_fingerprint == fingerprint
        && existing_meta.paper_count == paper_count
    {
        info!(
            count = paper_count,
            "Corpus unchanged ({} papers), skipping NLP analysis", paper_count
        );
        return;
    }

    let tfidf_results = resyn_core::nlp::tfidf::TfIdfEngine::compute_corpus(&extractions);

    for (arxiv_id, tfidf_vector) in &tfidf_results {
        let (top_terms, top_scores) =
            resyn_core::nlp::tfidf::TfIdfEngine::get_top_n(tfidf_vector, 5);

        let keywords_display: Vec<String> = top_terms
            .iter()
            .zip(top_scores.iter())
            .map(|(term, score)| format!("{term} ({score:.2})"))
            .collect();
        info!(
            paper = arxiv_id.as_str(),
            "Paper {}: {}",
            arxiv_id,
            keywords_display.join(", ")
        );

        let analysis = PaperAnalysis {
            arxiv_id: arxiv_id.clone(),
            tfidf_vector: tfidf_vector.clone(),
            top_terms,
            top_scores,
            analyzed_at: Utc::now().to_rfc3339(),
            corpus_fingerprint: fingerprint.clone(),
        };

        if let Err(err) = analysis_repo.upsert_analysis(&analysis).await {
            error!(paper_id = arxiv_id.as_str(), error = %err, "Failed to persist analysis");
        }
    }

    let metadata = AnalysisMetadata {
        key: "corpus_tfidf".to_string(),
        paper_count,
        corpus_fingerprint: fingerprint,
        last_analyzed: Utc::now().to_rfc3339(),
    };
    if let Err(err) = analysis_repo.upsert_metadata(&metadata).await {
        error!(error = %err, "Failed to persist analysis metadata");
    }

    let mut doc_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (_, tfidf_vector) in &tfidf_results {
        for term in tfidf_vector.keys() {
            *doc_freq.entry(term.clone()).or_insert(0) += 1;
        }
    }

    let mut df_pairs: Vec<(String, usize)> = doc_freq.into_iter().collect();
    df_pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    df_pairs.truncate(10);

    let corpus_terms_display: Vec<String> = df_pairs
        .iter()
        .map(|(term, count)| format!("{term} (in {count} papers)"))
        .collect();

    let avg_keywords = if tfidf_results.is_empty() {
        0.0
    } else {
        tfidf_results
            .iter()
            .map(|(_, v)| v.len().min(5))
            .sum::<usize>() as f64
            / tfidf_results.len() as f64
    };

    info!(
        papers_analyzed = paper_count,
        avg_keywords = avg_keywords,
        "NLP analysis complete: {} papers analyzed, avg {:.1} keywords/paper",
        paper_count,
        avg_keywords
    );
    info!("Top corpus terms: {}", corpus_terms_display.join(", "));
}

async fn run_llm_analysis(db: &Db, provider: &mut dyn LlmProvider) {
    let paper_repo = resyn_core::database::queries::PaperRepository::new(db);
    let llm_repo = LlmAnnotationRepository::new(db);

    let all_papers = paper_repo.get_all_papers().await.unwrap_or_else(|e| {
        error!(error = %e, "Failed to load papers for LLM analysis");
        std::process::exit(1);
    });

    let (mut annotated, mut skipped, mut failed) = (0usize, 0usize, 0usize);

    for paper in &all_papers {
        let id = resyn_core::utils::strip_version_suffix(&paper.id);
        if llm_repo.annotation_exists(&id).await.unwrap_or(false) {
            skipped += 1;
            continue;
        }
        match provider.annotate_paper(&id, &paper.summary).await {
            Ok(ann) => {
                if let Err(e) = llm_repo.upsert_annotation(&ann).await {
                    error!(paper_id = id.as_str(), error = %e, "Failed to persist LLM annotation");
                }
                annotated += 1;
            }
            Err(e) => {
                warn!(paper_id = id.as_str(), error = %e, "LLM annotation failed, skipping paper");
                failed += 1;
            }
        }
    }

    info!(
        annotated,
        skipped,
        failed,
        total = all_papers.len(),
        provider = provider.provider_name(),
        "LLM analysis: {}/{} papers annotated ({} cached, {} failed), provider: {}",
        annotated,
        all_papers.len(),
        skipped,
        failed,
        provider.provider_name()
    );
}

async fn run_gap_analysis(
    db: &Db,
    provider: &mut dyn LlmProvider,
    full_corpus: bool,
    verbose: bool,
) {
    let analysis_repo = AnalysisRepository::new(db);
    let llm_repo = LlmAnnotationRepository::new(db);
    let paper_repo = resyn_core::database::queries::PaperRepository::new(db);
    let gap_repo = GapFindingRepository::new(db);

    let analyses = match analysis_repo.get_all_analyses().await {
        Ok(a) => a,
        Err(e) => {
            warn!(error = %e, "Failed to load analyses for gap analysis");
            return;
        }
    };
    let annotations = match llm_repo.get_all_annotations().await {
        Ok(a) => a,
        Err(e) => {
            warn!(error = %e, "Failed to load annotations for gap analysis");
            return;
        }
    };

    if analyses.is_empty() || annotations.is_empty() {
        info!("No analyses/annotations found, skipping gap analysis");
        return;
    }

    let annotation_ids: Vec<String> = annotations.iter().map(|a| a.arxiv_id.clone()).collect();
    let fingerprint = resyn_core::nlp::tfidf::corpus_fingerprint(&annotation_ids);

    let skip_analysis =
        if let Ok(Some(existing_meta)) = analysis_repo.get_metadata("gap_analysis").await {
            existing_meta.corpus_fingerprint == fingerprint
        } else {
            false
        };

    if skip_analysis {
        info!("Gap corpus unchanged, skipping gap analysis");
    } else {
        let papers = match paper_repo.get_all_papers().await {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, "Failed to load papers for gap analysis graph");
                return;
            }
        };

        if full_corpus && papers.len() == annotation_ids.len() {
            info!("--full-corpus specified but no additional papers in DB beyond current crawl");
        }

        let graph = resyn_core::data_processing::graph_creation::create_graph_from_papers(&papers);

        let contradictions = resyn_core::gap_analysis::contradiction::find_contradictions(
            &analyses,
            &annotations,
            provider,
        )
        .await;

        let abc_bridges = resyn_core::gap_analysis::abc_bridge::find_abc_bridges(
            &analyses,
            &annotations,
            &graph,
            provider,
            full_corpus,
        )
        .await;

        for finding in contradictions.iter().chain(abc_bridges.iter()) {
            if let Err(e) = gap_repo.insert_gap_finding(finding).await {
                warn!(error = %e, "Failed to persist gap finding");
            }
        }

        let paper_count = annotation_ids.len() as u64;
        let metadata = AnalysisMetadata {
            key: "gap_analysis".to_string(),
            paper_count,
            corpus_fingerprint: fingerprint,
            last_analyzed: Utc::now().to_rfc3339(),
        };
        if let Err(e) = analysis_repo.upsert_metadata(&metadata).await {
            warn!(error = %e, "Failed to persist gap analysis metadata");
        }
    }

    let all_findings = match gap_repo.get_all_gap_findings().await {
        Ok(f) => f,
        Err(e) => {
            warn!(error = %e, "Failed to load gap findings for display");
            return;
        }
    };

    let table = resyn_core::gap_analysis::output::format_gap_table(&all_findings, verbose);
    print!("{table}");

    let summary = resyn_core::gap_analysis::output::format_gap_summary(&all_findings);
    info!("{summary}");
}
