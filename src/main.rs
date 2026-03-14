#[allow(dead_code)]
mod data_aggregation;
#[allow(dead_code)]
mod data_processing;
#[allow(dead_code)]
mod database;
#[allow(dead_code)]
mod datamodels;
#[allow(dead_code)]
mod error;
#[allow(dead_code)]
mod gap_analysis;
#[allow(dead_code)]
mod llm;
#[allow(dead_code)]
mod nlp;
#[allow(dead_code)]
mod utils;
#[allow(dead_code)]
mod validation;
#[allow(dead_code)]
mod visualization;

use chrono::Utc;
use clap::Parser;
use data_aggregation::arxiv_source::ArxivSource;
use data_aggregation::inspirehep_api::InspireHepClient;
use data_aggregation::traits::PaperSource;
use data_processing::graph_creation::create_graph_from_papers;
use database::client::Db;
use database::queries::{AnalysisRepository, GapFindingRepository, LlmAnnotationRepository};
use datamodels::analysis::{AnalysisMetadata, PaperAnalysis};
use datamodels::paper::Paper;
use eframe::run_native;
use llm::claude::ClaudeProvider;
use llm::noop::NoopProvider;
use llm::ollama::OllamaProvider;
use llm::traits::LlmProvider;
use tracing::{error, info, warn};
use visualization::force_graph_app::DemoApp;

#[derive(Parser, Debug)]
#[command(
    name = "resyn",
    about = "Research Synergy - Literature Based Discovery"
)]
struct Cli {
    /// arXiv paper ID to use as seed (e.g. 2503.18887)
    #[arg(short, long, default_value = "2503.18887")]
    paper_id: String,

    /// Maximum depth for recursive reference search
    #[arg(short = 'd', long, default_value_t = 3)]
    max_depth: usize,

    /// Rate limit delay between requests in seconds
    #[arg(short, long, default_value_t = 3)]
    rate_limit_secs: u64,

    /// Data source: "arxiv" or "inspirehep"
    #[arg(long, default_value = "arxiv")]
    source: String,

    /// Database connection string (e.g. "mem://", "surrealkv://./data")
    #[arg(long)]
    db: Option<String>,

    /// Skip crawling, load graph from database only
    #[arg(long, default_value_t = false)]
    db_only: bool,

    /// Run text extraction analysis after crawl and persist
    #[arg(long, default_value_t = false)]
    analyze: bool,

    /// Skip full-text extraction; all papers use abstract only
    #[arg(long, default_value_t = false)]
    skip_fulltext: bool,

    /// LLM provider for semantic extraction: claude, ollama, noop
    #[arg(long)]
    llm_provider: Option<String>,

    /// LLM model override (e.g. claude-sonnet-4-20250514, llama3.2)
    #[arg(long)]
    llm_model: Option<String>,

    /// Expand ABC-bridge scope to all papers in SurrealDB (not just current crawl)
    #[arg(long, default_value_t = false)]
    full_corpus: bool,

    /// Show full justifications in gap output table (default: truncated at 60 chars)
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = validation::validate_arxiv_id(&cli.paper_id) {
        error!(error = %e, "Invalid paper ID");
        std::process::exit(1);
    }

    // Connect to database if requested
    let db = if let Some(ref db_endpoint) = cli.db {
        match database::client::connect(db_endpoint).await {
            Ok(db) => {
                info!(endpoint = db_endpoint.as_str(), "Connected to database");
                Some(db)
            }
            Err(e) => {
                error!(error = %e, "Failed to connect to database");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // If db-only mode, load from database and skip crawling
    if cli.db_only {
        let Some(ref db) = db else {
            error!("--db-only requires --db to be specified");
            std::process::exit(1);
        };
        let repo = database::queries::PaperRepository::new(db);
        let (papers, _edges) = match repo.get_citation_graph(&cli.paper_id, cli.max_depth).await {
            Ok(result) => result,
            Err(e) => {
                error!(error = %e, "Failed to load graph from database");
                std::process::exit(1);
            }
        };
        info!(count = papers.len(), "Loaded papers from database");
        if cli.analyze {
            run_analysis(
                db,
                cli.rate_limit_secs,
                cli.skip_fulltext,
                cli.llm_provider.as_deref(),
                cli.llm_model.as_deref(),
                cli.full_corpus,
                cli.verbose,
            )
            .await;
        }
        launch_visualization(&papers);
        return;
    }

    // Create the appropriate paper source
    let client = utils::create_http_client();
    let mut source: Box<dyn PaperSource> = match cli.source.as_str() {
        "inspirehep" => {
            let inspire_client = InspireHepClient::new(client);
            Box::new(inspire_client)
        }
        _ => {
            let downloader = data_aggregation::html_parser::ArxivHTMLDownloader::new(client)
                .with_rate_limit(std::time::Duration::from_secs(cli.rate_limit_secs));
            Box::new(ArxivSource::new(downloader))
        }
    };

    // Run the BFS crawler
    let papers = data_aggregation::arxiv_utils::recursive_paper_search_by_references(
        &cli.paper_id,
        cli.max_depth,
        source.as_mut(),
    )
    .await;

    info!(count = papers.len(), "Recursive search completed");

    // Persist to database if connected
    if let Some(ref db) = db {
        let repo = database::queries::PaperRepository::new(db);
        for paper in &papers {
            if let Err(e) = repo.upsert_paper(paper).await {
                error!(paper_id = paper.id, error = %e, "Failed to upsert paper");
            }
        }
        for paper in &papers {
            if let Err(e) = repo.upsert_citations(paper).await {
                error!(paper_id = paper.id, error = %e, "Failed to upsert citations");
            }
        }
        info!(count = papers.len(), "Persisted papers to database");
    }

    // Run analysis pipeline if requested
    if cli.analyze {
        let Some(ref db) = db else {
            error!("--analyze requires --db to be specified");
            std::process::exit(1);
        };
        run_analysis(
            db,
            cli.rate_limit_secs,
            cli.skip_fulltext,
            cli.llm_provider.as_deref(),
            cli.llm_model.as_deref(),
            cli.full_corpus,
            cli.verbose,
        )
        .await;
    }

    launch_visualization(&papers);
}

async fn run_analysis(
    db: &Db,
    rate_limit_secs: u64,
    skip_fulltext: bool,
    llm_provider: Option<&str>,
    llm_model: Option<&str>,
    full_corpus: bool,
    verbose: bool,
) {
    let extraction_repo = database::queries::ExtractionRepository::new(db);
    let paper_repo = database::queries::PaperRepository::new(db);
    let all_papers = paper_repo.get_all_papers().await.unwrap_or_else(|e| {
        error!(error = %e, "Failed to load papers for analysis");
        std::process::exit(1);
    });

    let client = utils::create_http_client();
    let mut extractor = data_aggregation::text_extractor::Ar5ivExtractor::new(client)
        .with_rate_limit(std::time::Duration::from_secs(rate_limit_secs));

    let mut abstract_only_count: usize = 0;
    let mut skipped_count: usize = 0;
    for paper in &all_papers {
        let stripped_id = utils::strip_version_suffix(&paper.id);
        if extraction_repo
            .extraction_exists(&stripped_id)
            .await
            .unwrap_or(false)
        {
            skipped_count += 1;
            continue;
        }
        let result = if skip_fulltext {
            datamodels::extraction::TextExtractionResult::from_abstract(paper)
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

    // NLP analysis step runs after text extraction (locked decision: extract -> analyze order)
    // Reads from the text_extraction table (all papers in DB, not just current crawl)
    run_nlp_analysis(db).await;

    // LLM analysis step — only runs when --llm-provider is specified
    if let Some(provider_name) = llm_provider {
        let client = utils::create_http_client();
        let mut provider: Box<dyn LlmProvider> = match provider_name {
            "claude" => {
                let p = ClaudeProvider::new(client).unwrap_or_else(|e| {
                    error!(error = %e, "Failed to initialize Claude provider");
                    std::process::exit(1);
                });
                Box::new(if let Some(m) = llm_model {
                    p.with_model(m.to_string())
                } else {
                    p
                })
            }
            "ollama" => {
                let p = OllamaProvider::new(client);
                Box::new(if let Some(m) = llm_model {
                    p.with_model(m.to_string())
                } else {
                    p
                })
            }
            "noop" => Box::new(NoopProvider),
            other => {
                error!(provider = other, "Unknown LLM provider. Use: claude, ollama, noop");
                std::process::exit(1);
            }
        };
        run_llm_analysis(db, provider.as_mut()).await;
        run_gap_analysis(db, provider.as_mut(), full_corpus, verbose).await;
    }
}

async fn run_llm_analysis(db: &Db, provider: &mut dyn LlmProvider) {
    let paper_repo = database::queries::PaperRepository::new(db);
    let llm_repo = LlmAnnotationRepository::new(db);

    let all_papers = paper_repo.get_all_papers().await.unwrap_or_else(|e| {
        error!(error = %e, "Failed to load papers for LLM analysis");
        std::process::exit(1);
    });

    let (mut annotated, mut skipped, mut failed) = (0usize, 0usize, 0usize);

    for paper in &all_papers {
        let id = utils::strip_version_suffix(&paper.id);
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

async fn run_nlp_analysis(db: &Db) {
    let extraction_repo = database::queries::ExtractionRepository::new(db);
    let analysis_repo = database::queries::AnalysisRepository::new(db);

    // Step 1: Load ALL extractions from DB before any IDF computation (RESEARCH.md Pitfall 2)
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

    // Step 2: Corpus fingerprint check (INFR-02)
    let arxiv_ids: Vec<String> = extractions.iter().map(|e| e.arxiv_id.clone()).collect();
    let fingerprint = nlp::tfidf::corpus_fingerprint(&arxiv_ids);
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

    // Step 3: Compute TF-IDF for all documents (IDF is corpus-level, computed once)
    let tfidf_results = nlp::tfidf::TfIdfEngine::compute_corpus(&extractions);

    // Step 4: Extract top-5, persist, and log per-paper keywords
    for (arxiv_id, tfidf_vector) in &tfidf_results {
        let (top_terms, top_scores) = nlp::tfidf::TfIdfEngine::get_top_n(tfidf_vector, 5);

        // Build log string for top keywords
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

    // Step 5: Update corpus metadata
    let metadata = AnalysisMetadata {
        key: "corpus_tfidf".to_string(),
        paper_count,
        corpus_fingerprint: fingerprint,
        last_analyzed: Utc::now().to_rfc3339(),
    };
    if let Err(err) = analysis_repo.upsert_metadata(&metadata).await {
        error!(error = %err, "Failed to persist analysis metadata");
    }

    // Step 6: Corpus-level summary
    // Compute document frequencies for top corpus terms
    let mut doc_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (_, tfidf_vector) in &tfidf_results {
        for term in tfidf_vector.keys() {
            *doc_freq.entry(term.clone()).or_insert(0) += 1;
        }
    }

    // Top 10 corpus terms by document frequency
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

async fn run_gap_analysis(
    db: &Db,
    provider: &mut dyn LlmProvider,
    full_corpus: bool,
    verbose: bool,
) {
    let analysis_repo = AnalysisRepository::new(db);
    let llm_repo = LlmAnnotationRepository::new(db);
    let paper_repo = database::queries::PaperRepository::new(db);
    let gap_repo = GapFindingRepository::new(db);

    // Load all analyses and annotations
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

    // Corpus fingerprint cache guard
    let annotation_ids: Vec<String> = annotations.iter().map(|a| a.arxiv_id.clone()).collect();
    let fingerprint = nlp::tfidf::corpus_fingerprint(&annotation_ids);

    let skip_analysis = if let Ok(Some(existing_meta)) = analysis_repo.get_metadata("gap_analysis").await {
        existing_meta.corpus_fingerprint == fingerprint
    } else {
        false
    };

    if skip_analysis {
        info!("Gap corpus unchanged, skipping gap analysis");
    } else {
        // Build citation graph
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

        let graph = create_graph_from_papers(&papers);

        // Run contradiction detection
        let contradictions = gap_analysis::contradiction::find_contradictions(
            &analyses,
            &annotations,
            provider,
        )
        .await;

        // Run ABC-bridge discovery
        let abc_bridges = gap_analysis::abc_bridge::find_abc_bridges(
            &analyses,
            &annotations,
            &graph,
            provider,
            full_corpus,
        )
        .await;

        // Persist all findings
        for finding in contradictions.iter().chain(abc_bridges.iter()) {
            if let Err(e) = gap_repo.insert_gap_finding(finding).await {
                warn!(error = %e, "Failed to persist gap finding");
            }
        }

        // Update corpus metadata
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

    // Always display findings (including historical / cached)
    let all_findings = match gap_repo.get_all_gap_findings().await {
        Ok(f) => f,
        Err(e) => {
            warn!(error = %e, "Failed to load gap findings for display");
            return;
        }
    };

    let table = gap_analysis::output::format_gap_table(&all_findings, verbose);
    print!("{table}");

    let summary = gap_analysis::output::format_gap_summary(&all_findings);
    info!("{summary}");
}

fn launch_visualization(papers: &[Paper]) {
    let paper_graph = data_processing::graph_creation::create_graph_from_papers(papers);
    info!(
        nodes = paper_graph.node_count(),
        edges = paper_graph.edge_count(),
        "Graph created"
    );

    let graph_without_weights = paper_graph.map(|_, _| (), |_, _| ());

    let native_options = eframe::NativeOptions::default();

    run_native(
        "Paper graph interactive",
        native_options,
        Box::new(|cc| Ok(Box::new(DemoApp::new(cc, graph_without_weights)))),
    )
    .expect("failed to launch GUI");
}
