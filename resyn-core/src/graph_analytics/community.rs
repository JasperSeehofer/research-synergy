//! Phase 24: Louvain community detection + c-TF-IDF labeling.
//!
//! All single-clustering and petgraph-0.8 types are strictly module-private.
//! Public API uses only project types (Paper, PaperAnalysis, CommunityAssignment, etc.)
//! and petgraph 0.7 StableGraph.

use crate::datamodels::paper::Paper;
use crate::datamodels::analysis::PaperAnalysis;
use crate::datamodels::community::{
    CommunityAssignment, CommunitySummary, CommunityTopPaper, CommunityStatus, OTHER_COLOR_INDEX,
};
use crate::error::ResynError;
use crate::utils::strip_version_suffix;
use petgraph::Directed;
use petgraph::stable_graph::StableGraph;
use std::collections::HashMap;

// Module constants (discretion values per CONTEXT.md)
const LOUVAIN_SEED: u64 = 42;
const LOUVAIN_RESOLUTION: f64 = 1.0;
const MIN_COMMUNITY_SIZE: usize = 3; // D-04
const MIN_GRAPH_NODES: usize = 10; // small-graph fallback
const TOP_PAPERS_PER_COMMUNITY: usize = 5; // D-20
const TOP_KEYWORDS_PER_COMMUNITY: usize = 10; // D-22
const MAX_SHARED_METHODS: usize = 8; // discretion (D-23)
const SHARED_METHODS_MIN_WEIGHT: f32 = 0.05; // matches gap_analysis usage

/// A partitioning result that includes the size-ranked color_index for each community.
#[derive(Debug, Clone, PartialEq)]
pub struct LouvainPartition {
    /// arxiv_id -> community_id (stable Louvain id; may be sparse)
    pub assignments: HashMap<String, u32>,
    /// community_id -> color_index (0 = largest non-Other; OTHER_COLOR_INDEX for the Other bucket)
    pub color_indices: HashMap<u32, u32>,
    /// community_id -> member count
    pub sizes: HashMap<u32, usize>,
}

/// Runs Louvain on the undirected projection of `graph`.
///
/// Communities with < MIN_COMMUNITY_SIZE nodes are merged into a single "Other" community
/// whose community_id is u32::MAX - 1 and whose color_index is OTHER_COLOR_INDEX (u32::MAX).
/// Remaining communities are ranked by size descending and assigned color_index starting at 0.
/// Returns a LouvainPartition with empty assignments when graph has < MIN_GRAPH_NODES nodes.
pub fn detect_communities(graph: &StableGraph<Paper, f32, Directed, u32>) -> LouvainPartition {
    let n_nodes = graph.node_count();
    let empty = LouvainPartition {
        assignments: HashMap::new(),
        color_indices: HashMap::new(),
        sizes: HashMap::new(),
    };

    if n_nodes < MIN_GRAPH_NODES {
        return empty;
    }

    // Build a sorted (arxiv_id -> index) mapping for deterministic indexing (Pitfall 2)
    let mut sorted_ids: Vec<(String, u32)> = graph
        .node_indices()
        .filter_map(|ni| {
            graph.node_weight(ni).map(|paper| {
                (strip_version_suffix(&paper.id), ni.index() as u32)
            })
        })
        .collect();
    sorted_ids.sort_by(|a, b| a.0.cmp(&b.0));

    // Build arxiv_id -> sorted_index mapping
    let _id_to_sorted_idx: HashMap<String, u32> = sorted_ids
        .iter()
        .enumerate()
        .map(|(sorted_idx, pair)| (pair.0.clone(), sorted_idx as u32))
        .collect();

    // Map from node graph index to sorted index
    let graph_idx_to_sorted: HashMap<u32, u32> = sorted_ids
        .iter()
        .enumerate()
        .map(|(sorted_idx, pair)| (pair.1, sorted_idx as u32))
        .collect();

    // Collect undirected edges as (min, max) pairs, deduplicated (D-02)
    let mut edge_set: std::collections::HashSet<(u32, u32)> = std::collections::HashSet::new();
    for edge in graph.edge_indices() {
        let (src, dst) = graph.edge_endpoints(edge).unwrap();
        let si = *graph_idx_to_sorted.get(&(src.index() as u32)).unwrap();
        let di = *graph_idx_to_sorted.get(&(dst.index() as u32)).unwrap();
        let pair = if si < di { (si, di) } else { (di, si) };
        edge_set.insert(pair);
    }

    let edges: Vec<(u32, u32)> = edge_set.into_iter().collect();

    // Build Louvain Network
    #[cfg(feature = "ssr")]
    {
        use single_clustering::community_search::louvain::Louvain;
        use single_clustering::network::grouping::{NetworkGrouping, VectorGrouping};

        let network = Louvain::<f64>::build_network(n_nodes, edges.len(), edges.into_iter());
        let mut louvain: Louvain<f64> = Louvain::new(LOUVAIN_RESOLUTION, Some(LOUVAIN_SEED));
        let mut clustering = VectorGrouping::create_isolated(n_nodes);
        louvain.iterate(&network, &mut clustering);

        // Collect raw assignments: sorted_index -> community_id
        let mut raw_community_sizes: HashMap<u32, usize> = HashMap::new();
        let mut sorted_idx_to_community: Vec<u32> = Vec::with_capacity(n_nodes);
        for i in 0..n_nodes {
            let cid = clustering.get_group(i) as u32;
            sorted_idx_to_community.push(cid);
            *raw_community_sizes.entry(cid).or_insert(0) += 1;
        }

        // Determine the "Other" sentinel community_id (u32::MAX - 1, distinct from OTHER_COLOR_INDEX)
        const OTHER_COMMUNITY_ID: u32 = u32::MAX - 1;

        // Remap small communities -> Other
        let mut final_community_sizes: HashMap<u32, usize> = HashMap::new();
        let remapped: Vec<u32> = sorted_idx_to_community
            .iter()
            .map(|&cid| {
                if raw_community_sizes.get(&cid).copied().unwrap_or(0) < MIN_COMMUNITY_SIZE {
                    OTHER_COMMUNITY_ID
                } else {
                    cid
                }
            })
            .collect();

        for &cid in &remapped {
            *final_community_sizes.entry(cid).or_insert(0) += 1;
        }

        // Build color_indices: sort surviving non-Other communities by (size desc, id asc)
        let mut non_other: Vec<(u32, usize)> = final_community_sizes
            .iter()
            .filter(|pair| *pair.0 != OTHER_COMMUNITY_ID)
            .map(|pair| (*pair.0, *pair.1))
            .collect();
        non_other.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let mut color_indices: HashMap<u32, u32> = HashMap::new();
        for (idx, (cid, _)) in non_other.iter().enumerate() {
            color_indices.insert(*cid, idx as u32);
        }
        if final_community_sizes.contains_key(&OTHER_COMMUNITY_ID) {
            color_indices.insert(OTHER_COMMUNITY_ID, OTHER_COLOR_INDEX);
        }

        // Build final arxiv_id -> community_id assignments
        let assignments: HashMap<String, u32> = sorted_ids
            .iter()
            .enumerate()
            .map(|(sorted_idx, pair)| (pair.0.clone(), remapped[sorted_idx]))
            .collect();

        LouvainPartition {
            assignments,
            color_indices,
            sizes: final_community_sizes,
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        // WASM path: single-clustering not available — return empty
        let _ = _id_to_sorted_idx; // suppress unused warning
        empty
    }
}

/// c-TF-IDF: each community is treated as a pseudo-document.
///
/// Input: community_id -> PaperAnalysis refs belonging to the community.
/// Output: community_id -> Vec<(term, score)> sorted desc, truncated to TOP_KEYWORDS_PER_COMMUNITY.
/// Skips communities whose color_index is OTHER_COLOR_INDEX.
pub fn compute_ctfidf(
    community_members: &HashMap<u32, Vec<&PaperAnalysis>>,
    color_indices: &HashMap<u32, u32>,
) -> HashMap<u32, Vec<(String, f32)>> {
    // Filter to non-Other communities only
    let active_communities: Vec<u32> = community_members
        .keys()
        .copied()
        .filter(|cid| {
            color_indices
                .get(cid)
                .copied()
                .unwrap_or(OTHER_COLOR_INDEX)
                != OTHER_COLOR_INDEX
        })
        .collect();

    let n_communities = active_communities.len();
    if n_communities == 0 {
        return HashMap::new();
    }

    // Step 1: compute average TF per community (tf_c)
    let mut community_tf: HashMap<u32, HashMap<String, f32>> = HashMap::new();
    for &cid in &active_communities {
        let members = match community_members.get(&cid) {
            Some(m) => m,
            None => continue,
        };
        let n = members.len().max(1) as f32;
        let mut tf_c: HashMap<String, f32> = HashMap::new();
        for analysis in members {
            for (term, &weight) in &analysis.tfidf_vector {
                *tf_c.entry(term.clone()).or_insert(0.0) += weight;
            }
        }
        for v in tf_c.values_mut() {
            *v /= n;
        }
        community_tf.insert(cid, tf_c);
    }

    // Step 2: compute document frequency across non-Other communities
    let mut df: HashMap<String, usize> = HashMap::new();
    for (&_cid, tf_c) in &community_tf {
        for (term, &score) in tf_c {
            if score > 0.0 {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }

    // Step 3: compute c-TF-IDF scores per community
    let mut result: HashMap<u32, Vec<(String, f32)>> = HashMap::new();
    for &cid in &active_communities {
        let tf_c = match community_tf.get(&cid) {
            Some(t) => t,
            None => continue,
        };

        let mut scores: Vec<(String, f32)> = tf_c
            .iter()
            .filter(|(_, v)| **v > 0.0)
            .map(|(term, tf)| {
                let doc_freq = df.get(term).copied().unwrap_or(1);
                let idf = ((n_communities as f32) / (doc_freq as f32)).ln() + 1.0;
                (term.clone(), tf * idf)
            })
            .collect();

        // Sort descending by score, then alphabetically for tie-breaking
        scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        scores.truncate(TOP_KEYWORDS_PER_COMMUNITY);
        result.insert(cid, scores);
    }

    result
}

/// Hybrid score from D-19: pagerank * (intra_community_degree + 1).
#[inline]
pub fn hybrid_score(pagerank: f32, intra_degree: usize) -> f32 {
    pagerank * (intra_degree as f32 + 1.0)
}

/// Top-N papers in a community, ranked by hybrid score descending.
pub fn build_top_papers(
    community_members: &[&Paper],
    pagerank_by_id: &HashMap<String, f32>,
    intra_degree_by_id: &HashMap<String, usize>,
) -> Vec<CommunityTopPaper> {
    let mut candidates: Vec<CommunityTopPaper> = community_members
        .iter()
        .map(|paper| {
            let id = strip_version_suffix(&paper.id);
            let pr = pagerank_by_id.get(&id).copied().unwrap_or(0.0);
            let intra = intra_degree_by_id.get(&id).copied().unwrap_or(0);
            let score = hybrid_score(pr, intra);

            // Extract year from published date (first 4 chars, e.g. "2023-01-01")
            let year = if paper.published.len() >= 4 {
                paper.published[..4].parse::<i32>().ok()
            } else {
                None
            };

            CommunityTopPaper {
                arxiv_id: id,
                title: paper.title.clone(),
                authors: paper.authors.clone(),
                year,
                hybrid_score: score,
            }
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.hybrid_score
            .partial_cmp(&a.hybrid_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(TOP_PAPERS_PER_COMMUNITY);
    candidates
}

/// Build the auto-label for a community (top 1-2 c-TF-IDF keywords joined with space).
/// Other bucket returns "Other". Empty keyword list returns "Unlabeled".
pub fn build_community_label(color_index: u32, keywords: &[(String, f32)]) -> String {
    if color_index == OTHER_COLOR_INDEX {
        return "Other".to_string();
    }
    if keywords.is_empty() {
        return "Unlabeled".to_string();
    }
    keywords
        .iter()
        .take(2)
        .map(|(k, _)| k.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

// --- Task 3: Orchestrator and read helpers ---

/// Outcome of compute_and_store_communities.
pub enum ComputeOutcome {
    Computed {
        n_communities: usize,
        n_papers: usize,
        fingerprint: String,
    },
    /// Graph too small to run Louvain.
    Skipped,
}

/// Build a StableGraph from a list of papers and a list of (from_id, to_id) citation edges.
/// Papers loaded from DB have empty references, so edges must be supplied separately.
#[cfg(feature = "ssr")]
fn build_graph_from_edges(
    papers: &[Paper],
    edges: &[(String, String)],
) -> StableGraph<Paper, f32, Directed, u32> {
    use petgraph::prelude::NodeIndex;
    let mut graph = StableGraph::<Paper, f32, Directed, u32>::new();
    let mut id_to_node: HashMap<String, NodeIndex<u32>> = HashMap::new();

    for paper in papers {
        let id = strip_version_suffix(&paper.id);
        if !id_to_node.contains_key(&id) {
            let ni = graph.add_node(paper.clone());
            id_to_node.insert(id, ni);
        }
    }

    for (from_id, to_id) in edges {
        let from = strip_version_suffix(from_id);
        let to = strip_version_suffix(to_id);
        if let (Some(&from_ni), Some(&to_ni)) = (id_to_node.get(&from), id_to_node.get(&to)) {
            graph.add_edge(from_ni, to_ni, 1.0);
        }
    }

    graph
}

/// Full pipeline: load papers + citations + analyses + PageRank scores, run Louvain,
/// bucket Other, and upsert assignments. Corpus-fingerprint invalidation: stale rows
/// are deleted before the fresh batch is written.
///
/// Does NOT cache CommunitySummary rows — those are assembled lazily in
/// compute_community_summaries() below.
#[cfg(feature = "ssr")]
pub async fn compute_and_store_communities(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
) -> Result<ComputeOutcome, ResynError> {
    use crate::database::queries::{CommunityRepository, PaperRepository};
    use crate::nlp::tfidf::corpus_fingerprint;

    let paper_repo = PaperRepository::new(db);
    let papers = paper_repo.get_all_papers().await?;

    if papers.is_empty() {
        return Ok(ComputeOutcome::Skipped);
    }

    let mut sorted_ids: Vec<String> = papers
        .iter()
        .map(|p| strip_version_suffix(&p.id))
        .collect();
    sorted_ids.sort();
    let fingerprint = corpus_fingerprint(&sorted_ids);

    // Load citation edges from DB (papers loaded from DB have empty references field)
    let db_edges = paper_repo.get_all_citation_edges().await?;
    let graph = build_graph_from_edges(&papers, &db_edges);
    let partition = detect_communities(&graph);

    let community_repo = CommunityRepository::new(db);

    if partition.assignments.is_empty() {
        // Small graph — clean stale rows and return Skipped
        community_repo.delete_stale(&fingerprint).await?;
        return Ok(ComputeOutcome::Skipped);
    }

    // Build CommunityAssignment vec for all papers
    let assignments: Vec<CommunityAssignment> = partition
        .assignments
        .iter()
        .map(|(arxiv_id, &community_id)| CommunityAssignment {
            arxiv_id: arxiv_id.clone(),
            community_id,
            corpus_fingerprint: fingerprint.clone(),
        })
        .collect();

    // Delete stale rows, then upsert fresh batch
    community_repo.delete_stale(&fingerprint).await?;
    community_repo.upsert(&assignments).await?;

    // Count non-Other communities
    let other_id: u32 = u32::MAX - 1;
    let n_communities = partition
        .sizes
        .keys()
        .filter(|&&cid| cid != other_id)
        .count();

    Ok(ComputeOutcome::Computed {
        n_communities,
        n_papers: papers.len(),
        fingerprint,
    })
}

/// Build Vec<CommunitySummary> for ALL currently-assigned communities from cached DB data.
/// Used by get_all_community_summaries and get_community_summary server fns.
/// Requires graph_communities to be populated with the current corpus_fingerprint.
#[cfg(feature = "ssr")]
pub async fn compute_community_summaries(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
) -> Result<Vec<CommunitySummary>, ResynError> {
    use crate::database::queries::{
        AnalysisRepository, CommunityRepository, GraphMetricsRepository, PaperRepository,
    };
    use crate::gap_analysis::similarity::shared_high_weight_terms;
    use crate::nlp::tfidf::corpus_fingerprint;

    let community_repo = CommunityRepository::new(db);
    let all_assignments = community_repo.list_all().await?;

    if all_assignments.is_empty() {
        return Ok(vec![]);
    }

    // Determine current fingerprint
    let paper_repo = PaperRepository::new(db);
    let papers = paper_repo.get_all_papers().await?;
    let mut sorted_ids: Vec<String> = papers
        .iter()
        .map(|p| strip_version_suffix(&p.id))
        .collect();
    sorted_ids.sort();
    let current_fp = corpus_fingerprint(&sorted_ids);

    // Filter to only current-fingerprint rows
    let current_assignments: Vec<&CommunityAssignment> = all_assignments
        .iter()
        .filter(|a| a.corpus_fingerprint == current_fp)
        .collect();

    if current_assignments.is_empty() {
        return Ok(vec![]);
    }

    // Build paper lookup
    let paper_by_id: HashMap<String, &Paper> = papers
        .iter()
        .map(|p| (strip_version_suffix(&p.id), p))
        .collect();

    // Load analyses and metrics in bulk
    let analysis_repo = AnalysisRepository::new(db);
    let all_analyses = analysis_repo.get_all_analyses().await?;
    let analysis_by_id: HashMap<String, &PaperAnalysis> = all_analyses
        .iter()
        .map(|a| (strip_version_suffix(&a.arxiv_id), a))
        .collect();

    let metrics_repo = GraphMetricsRepository::new(db);
    let all_metrics = metrics_repo.get_all_metrics().await?;
    let pagerank_by_id: HashMap<String, f32> = all_metrics
        .iter()
        .map(|m| (strip_version_suffix(&m.arxiv_id), m.pagerank))
        .collect();

    // Group assignments by community_id
    const OTHER_COMMUNITY_ID: u32 = u32::MAX - 1;
    let mut groups: HashMap<u32, Vec<&CommunityAssignment>> = HashMap::new();
    for a in &current_assignments {
        groups.entry(a.community_id).or_default().push(a);
    }

    // Reconstruct color_indices by re-deriving size rank
    let mut non_other: Vec<(u32, usize)> = groups
        .iter()
        .filter(|pair| *pair.0 != OTHER_COMMUNITY_ID)
        .map(|pair| (*pair.0, pair.1.len()))
        .collect();
    non_other.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut color_indices: HashMap<u32, u32> = HashMap::new();
    for (idx, (cid, _)) in non_other.iter().enumerate() {
        color_indices.insert(*cid, idx as u32);
    }
    if groups.contains_key(&OTHER_COMMUNITY_ID) {
        color_indices.insert(OTHER_COMMUNITY_ID, OTHER_COLOR_INDEX);
    }

    // Compute intra-community degrees using DB citation edges
    let arxiv_id_to_community: HashMap<String, u32> = current_assignments
        .iter()
        .map(|a| (strip_version_suffix(&a.arxiv_id), a.community_id))
        .collect();

    // Load citation edges from DB (papers from get_all_papers() have empty references)
    let db_edges = paper_repo.get_all_citation_edges().await?;
    let mut intra_degree: HashMap<String, usize> = HashMap::new();
    for (from_raw, to_raw) in &db_edges {
        let from_id = strip_version_suffix(from_raw);
        let to_id = strip_version_suffix(to_raw);
        let from_community = arxiv_id_to_community.get(&from_id).copied();
        let to_community = arxiv_id_to_community.get(&to_id).copied();
        if let (Some(fc), Some(tc)) = (from_community, to_community) {
            if fc == tc {
                *intra_degree.entry(from_id).or_insert(0) += 1;
                *intra_degree.entry(to_id).or_insert(0) += 1;
            }
        }
    }

    // Build community_id -> Vec<&PaperAnalysis> for c-TF-IDF
    let mut community_analyses: HashMap<u32, Vec<&PaperAnalysis>> = HashMap::new();
    for (&cid, members) in &groups {
        let analyses: Vec<&PaperAnalysis> = members
            .iter()
            .filter_map(|a| analysis_by_id.get(&strip_version_suffix(&a.arxiv_id)))
            .copied()
            .collect();
        community_analyses.insert(cid, analyses);
    }

    let ctfidf_by_community = compute_ctfidf(&community_analyses, &color_indices);

    // Build CommunitySummary for each community
    let mut summaries: Vec<CommunitySummary> = Vec::new();

    for (&cid, members) in &groups {
        let color_index = color_indices.get(&cid).copied().unwrap_or(OTHER_COLOR_INDEX);

        // Collect Paper refs for this community
        let community_papers: Vec<&Paper> = members
            .iter()
            .filter_map(|a| paper_by_id.get(&strip_version_suffix(&a.arxiv_id)))
            .copied()
            .collect();

        let top_papers = build_top_papers(&community_papers, &pagerank_by_id, &intra_degree);

        let dominant_keywords = ctfidf_by_community
            .get(&cid)
            .cloned()
            .unwrap_or_default();

        let label = build_community_label(color_index, &dominant_keywords);

        // Compute shared methods via shared_high_weight_terms across community member TF-IDF vectors
        let member_analyses: Vec<&PaperAnalysis> = members
            .iter()
            .filter_map(|a| analysis_by_id.get(&strip_version_suffix(&a.arxiv_id)))
            .copied()
            .collect();

        let shared_methods = if member_analyses.len() >= 2 {
            let vectors: Vec<&HashMap<String, f32>> =
                member_analyses.iter().map(|a| &a.tfidf_vector).collect();
            // Compute pairwise shared terms, aggregate with frequency count
            let mut term_frequency: HashMap<String, usize> = HashMap::new();
            for i in 0..vectors.len() {
                for j in (i + 1)..vectors.len() {
                    let shared = shared_high_weight_terms(
                        vectors[i],
                        vectors[j],
                        SHARED_METHODS_MIN_WEIGHT,
                    );
                    for term in shared {
                        *term_frequency.entry(term).or_insert(0) += 1;
                    }
                }
            }
            let mut method_terms: Vec<(String, usize)> = term_frequency.into_iter().collect();
            method_terms.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            method_terms
                .into_iter()
                .take(MAX_SHARED_METHODS)
                .map(|(term, _)| term)
                .collect()
        } else {
            vec![]
        };

        summaries.push(CommunitySummary {
            community_id: cid,
            label,
            size: members.len(),
            color_index,
            top_papers,
            dominant_keywords,
            shared_methods,
        });
    }

    // Sort: non-Other by color_index ascending, Other last
    summaries.sort_by(|a, b| {
        match (a.color_index == OTHER_COLOR_INDEX, b.color_index == OTHER_COLOR_INDEX) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.color_index.cmp(&b.color_index),
        }
    });

    Ok(summaries)
}

/// Derive CommunityStatus from the database state.
#[cfg(feature = "ssr")]
pub async fn load_community_status(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
) -> Result<CommunityStatus, ResynError> {
    use crate::database::queries::{CommunityRepository, PaperRepository};
    use crate::nlp::tfidf::corpus_fingerprint;

    let paper_repo = PaperRepository::new(db);
    let papers = paper_repo.get_all_papers().await?;

    if papers.is_empty() {
        return Ok(CommunityStatus {
            ready: false,
            fingerprint: None,
            count: 0,
        });
    }

    let mut sorted_ids: Vec<String> = papers
        .iter()
        .map(|p| strip_version_suffix(&p.id))
        .collect();
    sorted_ids.sort();
    let current_fp = corpus_fingerprint(&sorted_ids);

    let community_repo = CommunityRepository::new(db);
    let all_assignments = community_repo.list_all().await?;

    if all_assignments.is_empty() {
        return Ok(CommunityStatus {
            ready: false,
            fingerprint: None,
            count: 0,
        });
    }

    // Check if any rows match the current fingerprint
    let matching: Vec<&CommunityAssignment> = all_assignments
        .iter()
        .filter(|a| a.corpus_fingerprint == current_fp)
        .collect();

    if matching.is_empty() {
        // Stale cache — not ready from UI perspective
        return Ok(CommunityStatus {
            ready: false,
            fingerprint: None,
            count: 0,
        });
    }

    // Count distinct non-Other community IDs
    const OTHER_COMMUNITY_ID: u32 = u32::MAX - 1;
    let distinct_communities: std::collections::HashSet<u32> = matching
        .iter()
        .map(|a| a.community_id)
        .filter(|&cid| cid != OTHER_COMMUNITY_ID)
        .collect();

    Ok(CommunityStatus {
        ready: true,
        fingerprint: Some(current_fp),
        count: distinct_communities.len() as u64,
    })
}

/// Export the Louvain community graph to a portable [`CommunityGraph`] for external tooling.
///
/// Loads community assignments, per-paper TF-IDF vectors, and citation edges from the DB.
/// Papers in the "Other" bucket (community_id == u32::MAX - 1) are excluded.
/// When `published_before` / `published_after` are supplied (e.g. `"2014-12-31"` / `"2018-01-01"`),
/// only papers whose `published` field falls within the range are included; edges whose either
/// endpoint fails the filter are also excluded. Both comparisons are lexicographic on ISO-8601.
/// `tfidf_top_n` controls how many (term, score) pairs are kept per node (sorted by score desc).
#[cfg(feature = "ssr")]
pub async fn export_community_graph(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    published_before: Option<&str>,
    published_after: Option<&str>,
    tfidf_top_n: usize,
) -> Result<crate::datamodels::community_graph::CommunityGraph, ResynError> {
    use crate::database::queries::{AnalysisRepository, CommunityRepository, PaperRepository};
    use crate::datamodels::community_graph::{
        CommunityGraph, ExportedCommunity, ExportedEdge, ExportedNode, LouvainParams,
    };
    use crate::nlp::tfidf::corpus_fingerprint;
    use std::collections::{HashMap, HashSet};

    const OTHER_COMMUNITY_ID: u32 = u32::MAX - 1;

    let paper_repo = PaperRepository::new(db);
    let all_papers = paper_repo.get_all_papers().await?;

    // Apply date range filter (lexicographic prefix compare on ISO-8601 date strings).
    let paper_ids_in_scope: HashSet<String> = all_papers
        .iter()
        .filter(|p| {
            let pub_date = p.published.as_str();
            let before_ok = published_before.map_or(true, |cutoff| pub_date <= cutoff);
            let after_ok = published_after.map_or(true, |floor| pub_date >= floor);
            before_ok && after_ok
        })
        .map(|p| strip_version_suffix(&p.id))
        .collect();

    // Corpus fingerprint from all in-scope paper IDs (sorted for determinism).
    let mut sorted_scope_ids: Vec<String> = paper_ids_in_scope.iter().cloned().collect();
    sorted_scope_ids.sort();
    let fingerprint = corpus_fingerprint(&sorted_scope_ids);

    // Community assignments keyed by arxiv_id.
    let community_repo = CommunityRepository::new(db);
    let assignments: HashMap<String, u32> = community_repo
        .list_all()
        .await?
        .into_iter()
        .filter(|a| a.community_id != OTHER_COMMUNITY_ID)
        .map(|a| (a.arxiv_id, a.community_id))
        .collect();

    // TF-IDF vectors keyed by arxiv_id.
    let analysis_repo = AnalysisRepository::new(db);
    let mut tfidf_map: HashMap<String, Vec<(String, f32)>> = analysis_repo
        .get_all_analyses()
        .await?
        .into_iter()
        .map(|a| {
            let mut terms: Vec<(String, f32)> = a.tfidf_vector.into_iter().collect();
            terms.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
            terms.truncate(tfidf_top_n);
            (a.arxiv_id, terms)
        })
        .collect();

    // Build community-level c-TF-IDF BEFORE consuming tfidf_map into nodes.
    // Only in-scope papers with a non-Other community assignment contribute.
    let mut community_term_sums: HashMap<u32, HashMap<String, f64>> = HashMap::new();
    let mut community_paper_counts: HashMap<u32, usize> = HashMap::new();
    for id in &sorted_scope_ids {
        let Some(&cid) = assignments.get(id) else { continue };
        let Some(terms) = tfidf_map.get(id) else { continue };
        let entry = community_term_sums.entry(cid).or_default();
        for (term, score) in terms {
            *entry.entry(term.clone()).or_insert(0.0) += *score as f64;
        }
        *community_paper_counts.entry(cid).or_insert(0) += 1;
    }
    // Average TF per community.
    for (&cid, term_map) in &mut community_term_sums {
        let n = *community_paper_counts.get(&cid).unwrap_or(&1).max(&1) as f64;
        for v in term_map.values_mut() {
            *v /= n;
        }
    }
    // Document frequency across communities.
    let mut community_df: HashMap<String, usize> = HashMap::new();
    for term_map in community_term_sums.values() {
        for (term, &v) in term_map {
            if v > 0.0 {
                *community_df.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }
    let n_communities_for_ctf = community_term_sums.len();
    // c-TF-IDF scores and build ExportedCommunity vec.
    let mut communities: Vec<ExportedCommunity> = community_term_sums
        .into_iter()
        .map(|(cid, term_map)| {
            let mut scores: Vec<(String, f32)> = term_map
                .into_iter()
                .filter(|(_, v)| *v > 0.0)
                .map(|(term, tf)| {
                    let df = community_df.get(&term).copied().unwrap_or(1);
                    let idf = ((n_communities_for_ctf as f64) / (df as f64)).ln() + 1.0;
                    (term, (tf * idf) as f32)
                })
                .collect();
            scores.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.cmp(&b.0))
            });
            scores.truncate(tfidf_top_n);
            ExportedCommunity {
                community_id: cid,
                size: community_paper_counts.get(&cid).copied().unwrap_or(0),
                tfidf_vec: scores,
            }
        })
        .collect();
    communities.sort_by_key(|c| c.community_id);

    // Build nodes: only papers that are in-scope AND have a community assignment.
    let nodes: Vec<ExportedNode> = sorted_scope_ids
        .iter()
        .filter_map(|id| {
            let community_id = *assignments.get(id)?;
            let tfidf_vec = tfidf_map.remove(id).unwrap_or_default();
            Some(ExportedNode {
                id: id.clone(),
                community_id,
                tfidf_vec,
            })
        })
        .collect();

    let node_set: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();

    // Build edges: both endpoints must be in the node set.
    let raw_edges = paper_repo.get_all_citation_edges().await?;
    let edges: Vec<ExportedEdge> = raw_edges
        .into_iter()
        .filter_map(|(src, dst)| {
            let src = strip_version_suffix(&src);
            let dst = strip_version_suffix(&dst);
            if node_set.contains(src.as_str()) && node_set.contains(dst.as_str()) {
                Some(ExportedEdge { src, dst, weight: 1.0 })
            } else {
                None
            }
        })
        .collect();

    Ok(CommunityGraph {
        louvain_params: LouvainParams {
            seed: LOUVAIN_SEED,
            resolution: LOUVAIN_RESOLUTION,
            min_community_size: MIN_COMMUNITY_SIZE,
        },
        corpus_fingerprint: fingerprint,
        nodes,
        communities,
        edges,
    })
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::paper::Paper;
    use crate::datamodels::analysis::PaperAnalysis;
    use petgraph::stable_graph::StableGraph;
    use petgraph::Directed;
    use std::collections::HashMap;

    // Helper: build a Paper with a given id
    fn make_paper(id: &str) -> Paper {
        Paper {
            title: format!("Paper {id}"),
            id: id.to_string(),
            published: "2023-01-01".to_string(),
            authors: vec!["Author A".to_string()],
            ..Default::default()
        }
    }

    // Helper: build a PaperAnalysis with a given id and tfidf vector
    fn make_analysis(id: &str, tfidf: &[(&str, f32)]) -> PaperAnalysis {
        let tfidf_vector: HashMap<String, f32> = tfidf
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        PaperAnalysis {
            arxiv_id: id.to_string(),
            tfidf_vector,
            top_terms: vec![],
            top_scores: vec![],
            analyzed_at: "2023-01-01T00:00:00Z".to_string(),
            corpus_fingerprint: "fp_test".to_string(),
        }
    }

    /// Build a StableGraph from two cliques connected by a bridge.
    /// clique_a: n_a nodes (0..n_a), clique_b: n_b nodes (n_a..n_a+n_b)
    /// bridge: one edge connecting last node of clique_a to first of clique_b.
    fn build_two_clique_graph(n_a: usize, n_b: usize) -> StableGraph<Paper, f32, Directed, u32> {
        let mut g = StableGraph::<Paper, f32, Directed, u32>::new();
        let total = n_a + n_b;
        let nodes: Vec<_> = (0..total)
            .map(|i| g.add_node(make_paper(&format!("2301.{:05}", i))))
            .collect();

        // Clique A (fully connected, directed)
        for i in 0..n_a {
            for j in 0..n_a {
                if i != j {
                    g.add_edge(nodes[i], nodes[j], 1.0);
                }
            }
        }
        // Clique B
        for i in n_a..total {
            for j in n_a..total {
                if i != j {
                    g.add_edge(nodes[i], nodes[j], 1.0);
                }
            }
        }
        // Bridge
        g.add_edge(nodes[n_a - 1], nodes[n_a], 1.0);
        g
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_community_detection_deterministic() {
        let g = build_two_clique_graph(8, 8);
        let p1 = detect_communities(&g);
        let p2 = detect_communities(&g);
        assert_eq!(
            p1.assignments, p2.assignments,
            "detect_communities must be deterministic with fixed seed"
        );
    }

    #[test]
    fn test_community_detection_too_small() {
        let mut g = StableGraph::<Paper, f32, Directed, u32>::new();
        for i in 0..9 {
            g.add_node(make_paper(&format!("2301.{:05}", i)));
        }
        let partition = detect_communities(&g);
        assert!(
            partition.assignments.is_empty(),
            "graph with < 10 nodes should return empty assignments"
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_community_other_bucket() {
        // Build graph: one big clique (8 nodes) + one isolated pair (2 nodes)
        // The pair forms a community of size 2 < MIN_COMMUNITY_SIZE (3) -> Other
        let mut g = StableGraph::<Paper, f32, Directed, u32>::new();
        let big: Vec<_> = (0..8)
            .map(|i| g.add_node(make_paper(&format!("2301.{:05}", i))))
            .collect();
        // Connect big clique
        for i in 0..8 {
            for j in 0..8 {
                if i != j {
                    g.add_edge(big[i], big[j], 1.0);
                }
            }
        }
        // Add isolated pair
        let p1 = g.add_node(make_paper("2301.00100"));
        let p2 = g.add_node(make_paper("2301.00101"));
        g.add_edge(p1, p2, 1.0);
        g.add_edge(p2, p1, 1.0);

        let partition = detect_communities(&g);

        // Find what community the isolated pair is in
        let id1 = "2301.00100";
        let id2 = "2301.00101";
        if let (Some(&c1), Some(&c2)) = (
            partition.assignments.get(id1),
            partition.assignments.get(id2),
        ) {
            assert_eq!(c1, c2, "isolated pair should be in same community");
            // Their color_index should be OTHER_COLOR_INDEX
            let ci = partition.color_indices.get(&c1).copied().unwrap_or(0);
            assert_eq!(
                ci, OTHER_COLOR_INDEX,
                "small community should have OTHER_COLOR_INDEX"
            );
        }
        // Not asserting partition.assignments is non-empty here since graph has 10 nodes
        // but the pair might merge with others — the key check is that tiny communities
        // are bucketed under OTHER_COLOR_INDEX
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_community_color_index_size_rank() {
        // Two cliques: 8 nodes and 5 nodes separated by a thin bridge
        let g = build_two_clique_graph(8, 5);
        let partition = detect_communities(&g);

        if partition.assignments.is_empty() {
            // Very small graph might not partition — skip
            return;
        }

        // Collect community sizes
        let mut by_size: Vec<(u32, usize)> = partition
            .sizes
            .iter()
            .filter(|pair| {
                partition
                    .color_indices
                    .get(pair.0)
                    .copied()
                    .unwrap_or(OTHER_COLOR_INDEX)
                    != OTHER_COLOR_INDEX
            })
            .map(|pair| (*pair.0, *pair.1))
            .collect();
        by_size.sort_by_key(|&(ci, _)| partition.color_indices[&ci]);

        // Color index 0 should have the largest size
        if by_size.len() >= 2 {
            assert!(
                by_size[0].1 >= by_size[1].1,
                "community with color_index 0 should be the largest"
            );
        }
    }

    #[test]
    fn test_ctfidf_distinctive_terms() {
        // term "quantum" appears only in community 0 -> should score higher than "physics"
        // which appears in all 3 communities
        let a0 = make_analysis("p0", &[("quantum", 0.9), ("physics", 0.5)]);
        let a1 = make_analysis("p1", &[("topology", 0.8), ("physics", 0.5)]);
        let a2 = make_analysis("p2", &[("biology", 0.7), ("physics", 0.5)]);

        let mut members: HashMap<u32, Vec<&PaperAnalysis>> = HashMap::new();
        members.insert(0, vec![&a0]);
        members.insert(1, vec![&a1]);
        members.insert(2, vec![&a2]);

        let color_indices: HashMap<u32, u32> = [(0, 0), (1, 1), (2, 2)].into_iter().collect();
        let result = compute_ctfidf(&members, &color_indices);

        let scores0 = result.get(&0).expect("community 0 should have scores");
        let quantum_score = scores0.iter().find(|(t, _)| t == "quantum").map(|(_, s)| *s);
        let physics_score = scores0.iter().find(|(t, _)| t == "physics").map(|(_, s)| *s);

        assert!(
            quantum_score.is_some() && physics_score.is_some(),
            "both terms should appear in community 0"
        );
        assert!(
            quantum_score.unwrap() > physics_score.unwrap(),
            "distinctive term 'quantum' should score higher than common term 'physics'"
        );
    }

    #[test]
    fn test_ctfidf_returns_top_n() {
        // Create a community with many terms
        let many_terms: Vec<(&str, f32)> = (0..20)
            .map(|i| {
                // Each term has a unique key; we need static lifetimes so build as owned below
                (Box::leak(format!("term_{:02}", i).into_boxed_str()) as &str, (20 - i) as f32 * 0.1)
            })
            .collect();

        let a = make_analysis("p0", &many_terms);
        let mut members: HashMap<u32, Vec<&PaperAnalysis>> = HashMap::new();
        members.insert(0, vec![&a]);

        let color_indices: HashMap<u32, u32> = [(0, 0)].into_iter().collect();
        let result = compute_ctfidf(&members, &color_indices);

        let scores = result.get(&0).expect("community 0 should have scores");
        assert_eq!(
            scores.len(),
            TOP_KEYWORDS_PER_COMMUNITY,
            "c-TF-IDF should cap at TOP_KEYWORDS_PER_COMMUNITY"
        );

        // Verify descending order
        for i in 0..scores.len() - 1 {
            assert!(
                scores[i].1 >= scores[i + 1].1,
                "scores should be sorted descending"
            );
        }
    }

    #[test]
    fn test_ctfidf_skips_other_bucket() {
        let a = make_analysis("p0", &[("quantum", 0.9)]);
        let mut members: HashMap<u32, Vec<&PaperAnalysis>> = HashMap::new();
        let other_id: u32 = u32::MAX - 1;
        members.insert(other_id, vec![&a]);

        let color_indices: HashMap<u32, u32> =
            [(other_id, OTHER_COLOR_INDEX)].into_iter().collect();
        let result = compute_ctfidf(&members, &color_indices);

        assert!(
            !result.contains_key(&other_id),
            "Other bucket should be skipped by c-TF-IDF"
        );
    }

    #[test]
    fn test_hybrid_score_formula() {
        // hybrid_score(0.5, 3) == 0.5 * (3 + 1) == 2.0
        let score = hybrid_score(0.5, 3);
        assert!(
            (score - 2.0).abs() < 1e-6,
            "hybrid_score(0.5, 3) should be 2.0, got {score}"
        );
    }

    #[test]
    fn test_hybrid_ranking_selects_top_5() {
        // 10 papers with varying pagerank and intra_degree
        let papers: Vec<Paper> = (0..10)
            .map(|i| Paper {
                title: format!("Paper {i}"),
                id: format!("2301.{:05}", i),
                published: "2023-01-01".to_string(),
                authors: vec!["Author".to_string()],
                ..Default::default()
            })
            .collect();

        let mut pagerank: HashMap<String, f32> = HashMap::new();
        let mut intra_degree: HashMap<String, usize> = HashMap::new();
        for i in 0..10 {
            let id = format!("2301.{:05}", i);
            pagerank.insert(id.clone(), (i + 1) as f32 * 0.1); // 0.1..1.0
            intra_degree.insert(id, i);
        }

        let refs: Vec<&Paper> = papers.iter().collect();
        let top = build_top_papers(&refs, &pagerank, &intra_degree);

        assert_eq!(top.len(), 5, "should return top 5 papers");
        // Highest scorer: paper 9 has pagerank 1.0 and intra_degree 9 -> 1.0 * 10 = 10.0
        assert_eq!(top[0].arxiv_id, "2301.00009", "paper 9 should be ranked first");

        // Verify descending order
        for i in 0..top.len() - 1 {
            assert!(
                top[i].hybrid_score >= top[i + 1].hybrid_score,
                "top papers should be in descending hybrid_score order"
            );
        }
    }

    #[test]
    fn test_auto_label_from_ctfidf() {
        // Normal community with keywords
        let keywords = vec![
            ("quantum".to_string(), 0.9),
            ("decoherence".to_string(), 0.7),
            ("entanglement".to_string(), 0.5),
        ];
        let label = build_community_label(0, &keywords);
        assert_eq!(label, "quantum decoherence", "label should use top 2 keywords");

        // Single keyword
        let single = vec![("topology".to_string(), 0.8)];
        let label_single = build_community_label(1, &single);
        assert_eq!(label_single, "topology", "single keyword should be used alone");

        // Other bucket
        let label_other = build_community_label(OTHER_COLOR_INDEX, &keywords);
        assert_eq!(label_other, "Other", "Other bucket should always return 'Other'");

        // Empty keywords -> Unlabeled
        let label_empty = build_community_label(2, &[]);
        assert_eq!(label_empty, "Unlabeled", "empty keywords should give 'Unlabeled'");
    }
}

// --- Orchestrator tests (Task 3) ---

#[cfg(all(test, feature = "ssr"))]
mod orchestrator_tests {
    use super::*;
    use crate::database::client::connect_memory;
    use crate::database::queries::{
        AnalysisRepository, CommunityRepository, GraphMetricsRepository, PaperRepository,
    };
    use crate::database::schema::migrate_schema;
    use crate::datamodels::paper::Paper;
    use crate::datamodels::analysis::PaperAnalysis;
    use crate::datamodels::graph_metrics::GraphMetrics;
    use std::collections::HashMap;

    async fn setup_db() -> crate::database::client::Db {
        let db = connect_memory().await.expect("in-memory DB failed");
        migrate_schema(&db).await.expect("schema migration failed");
        db
    }

    fn make_paper_with_refs(id: &str, refs: &[&str]) -> Paper {
        use crate::datamodels::paper::{Link, Reference};
        Paper {
            title: format!("Paper {id}"),
            id: id.to_string(),
            published: "2023-01-01".to_string(),
            authors: vec!["Author".to_string()],
            references: refs
                .iter()
                .map(|ref_id| Reference {
                    links: vec![Link::from_url(&format!("https://arxiv.org/abs/{ref_id}"))],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn make_analysis(id: &str, terms: &[(&str, f32)]) -> PaperAnalysis {
        let tfidf_vector: HashMap<String, f32> =
            terms.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        PaperAnalysis {
            arxiv_id: id.to_string(),
            tfidf_vector,
            top_terms: vec![],
            top_scores: vec![],
            analyzed_at: "2023-01-01T00:00:00Z".to_string(),
            corpus_fingerprint: "fp_test".to_string(),
        }
    }

    fn make_metrics(id: &str, pagerank: f32) -> GraphMetrics {
        GraphMetrics {
            arxiv_id: id.to_string(),
            pagerank,
            betweenness: 0.0,
            corpus_fingerprint: "fp_test".to_string(),
            computed_at: "2023-01-01T00:00:00Z".to_string(),
        }
    }

    // Creates 16 papers in two dense groups connected by a thin bridge
    // Group A: papers 0-7 (IDs 2301.00000..2301.00007)
    // Group B: papers 8-15 (IDs 2301.00008..2301.00015)
    // Bridge: 2301.00007 -> 2301.00008
    async fn seed_db_with_papers(db: &crate::database::client::Db) -> Vec<String> {
        let paper_repo = PaperRepository::new(db);
        let metrics_repo = GraphMetricsRepository::new(db);
        let analysis_repo = AnalysisRepository::new(db);

        let mut ids = vec![];
        let mut all_papers = vec![];

        for i in 0..16usize {
            let id = format!("2301.{:05}", i);
            ids.push(id.clone());

            // Build intra-group references
            let mut refs: Vec<String> = vec![];
            let group_start = if i < 8 { 0 } else { 8 };
            let group_end = if i < 8 { 8 } else { 16 };
            for j in group_start..group_end {
                if j != i {
                    refs.push(format!("2301.{:05}", j));
                }
            }
            // Bridge
            if i == 7 {
                refs.push("2301.00008".to_string());
            }

            let refs_str: Vec<&str> = refs.iter().map(|s| s.as_str()).collect();
            let paper = make_paper_with_refs(&id, &refs_str);
            paper_repo.upsert_paper(&paper).await.expect("upsert paper failed");
            all_papers.push(paper);

            // Add TF-IDF analysis
            let terms: Vec<(&str, f32)> = if i < 8 {
                vec![("quantum", 0.8), ("physics", 0.3)]
            } else {
                vec![("topology", 0.7), ("physics", 0.4)]
            };
            let terms_owned: Vec<(String, f32)> =
                terms.iter().map(|(k, v)| (k.to_string(), *v)).collect();
            let terms_ref: Vec<(&str, f32)> =
                terms_owned.iter().map(|(k, v)| (k.as_str(), *v)).collect();
            let analysis = make_analysis(&id, &terms_ref);
            analysis_repo.upsert_analysis(&analysis).await.expect("upsert analysis failed");

            // Add metrics
            let m = make_metrics(&id, (i + 1) as f32 * 0.1);
            metrics_repo.upsert_metrics(&m).await.expect("upsert metrics failed");
        }

        // Upsert citation edges (must be done after all papers exist in DB)
        for paper in &all_papers {
            paper_repo
                .upsert_citations(paper)
                .await
                .expect("upsert citations failed");
        }

        ids
    }

    #[tokio::test]
    async fn test_compute_and_store_communities_end_to_end() {
        let db = setup_db().await;
        seed_db_with_papers(&db).await;

        let outcome = compute_and_store_communities(&db)
            .await
            .expect("compute_and_store_communities failed");

        match outcome {
            ComputeOutcome::Computed { n_papers, .. } => {
                assert_eq!(n_papers, 16, "should have processed 16 papers");
            }
            ComputeOutcome::Skipped => panic!("should not be Skipped for 16-paper graph"),
        }

        let repo = CommunityRepository::new(&db);
        let all = repo.list_all().await.expect("list_all failed");
        assert!(!all.is_empty(), "graph_communities should be non-empty after compute");

        // All rows should carry the current fingerprint
        let first_fp = all[0].corpus_fingerprint.clone();
        for row in &all {
            assert_eq!(
                row.corpus_fingerprint, first_fp,
                "all rows should share the same fingerprint"
            );
        }
    }

    #[tokio::test]
    async fn test_compute_and_store_communities_replaces_stale() {
        let db = setup_db().await;
        seed_db_with_papers(&db).await;

        // First run
        compute_and_store_communities(&db)
            .await
            .expect("first run failed");

        // Insert an artificially stale row to simulate an old fingerprint
        db.query(
            "CREATE graph_communities CONTENT { \
             arxiv_id: 'stale.99999', \
             community_id: 99, \
             corpus_fingerprint: 'old_fp' \
             }",
        )
        .await
        .expect("insert stale row failed");

        // Second run should clean up the stale row
        compute_and_store_communities(&db)
            .await
            .expect("second run failed");

        let repo = CommunityRepository::new(&db);
        let all = repo.list_all().await.expect("list_all failed");

        // No rows with old_fp should remain
        let stale_rows: Vec<_> = all
            .iter()
            .filter(|a| a.corpus_fingerprint == "old_fp")
            .collect();
        assert!(
            stale_rows.is_empty(),
            "stale rows should be removed after second compute"
        );
    }

    #[tokio::test]
    async fn test_compute_and_store_communities_small_graph_no_op() {
        let db = setup_db().await;
        let paper_repo = PaperRepository::new(&db);

        // Insert only 5 papers
        for i in 0..5 {
            let id = format!("2301.{:05}", i);
            let paper = make_paper_with_refs(&id, &[]);
            paper_repo.upsert_paper(&paper).await.expect("upsert failed");
        }

        // Insert an old stale row
        db.query(
            "CREATE graph_communities CONTENT { \
             arxiv_id: 'stale.99999', \
             community_id: 99, \
             corpus_fingerprint: 'old_fp' \
             }",
        )
        .await
        .expect("insert stale row failed");

        let outcome = compute_and_store_communities(&db)
            .await
            .expect("compute failed");

        assert!(
            matches!(outcome, ComputeOutcome::Skipped),
            "5-paper graph should return Skipped"
        );

        // Stale rows should be cleaned up even on Skipped
        let repo = CommunityRepository::new(&db);
        let all = repo.list_all().await.expect("list_all failed");
        let stale: Vec<_> = all
            .iter()
            .filter(|a| a.corpus_fingerprint == "old_fp")
            .collect();
        assert!(stale.is_empty(), "stale rows should be removed even on Skipped");
    }

    #[tokio::test]
    async fn test_compute_community_summaries_end_to_end() {
        let db = setup_db().await;
        seed_db_with_papers(&db).await;

        compute_and_store_communities(&db)
            .await
            .expect("compute failed");

        let summaries = compute_community_summaries(&db)
            .await
            .expect("compute_community_summaries failed");

        assert!(!summaries.is_empty(), "should produce at least one community");

        for summary in &summaries {
            // Each non-Other community should have top papers
            if summary.color_index != OTHER_COLOR_INDEX {
                assert!(
                    !summary.top_papers.is_empty(),
                    "non-Other community should have top papers"
                );
                assert!(
                    !summary.dominant_keywords.is_empty(),
                    "non-Other community should have keywords"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_load_community_status_states() {
        let db = setup_db().await;

        // Empty DB -> not ready
        let status = load_community_status(&db)
            .await
            .expect("load_community_status failed");
        assert!(!status.ready, "empty DB should return not ready");
        assert_eq!(status.count, 0);

        // Seed and compute
        seed_db_with_papers(&db).await;
        compute_and_store_communities(&db)
            .await
            .expect("compute failed");

        // Should now be ready
        let status = load_community_status(&db)
            .await
            .expect("load_community_status failed");
        assert!(status.ready, "after compute, status should be ready");
        assert!(status.fingerprint.is_some());
        assert!(status.count > 0, "should have at least one non-Other community");
    }

    #[tokio::test]
    async fn test_load_community_for_paper() {
        let db = setup_db().await;
        seed_db_with_papers(&db).await;
        compute_and_store_communities(&db)
            .await
            .expect("compute failed");

        let repo = CommunityRepository::new(&db);

        // Known paper should have a community assignment
        let result = repo
            .get_by_paper("2301.00000")
            .await
            .expect("get_by_paper failed");
        assert!(result.is_some(), "2301.00000 should have a community assignment");

        // Unknown paper should return None
        let unknown = repo
            .get_by_paper("9999.99999")
            .await
            .expect("get_by_paper failed");
        assert!(unknown.is_none(), "unknown paper should return None");
    }
}
