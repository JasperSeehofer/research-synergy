use leptos::prelude::*;

// Re-export community types so the client side can use them without depending on resyn-core directly.
#[cfg(feature = "ssr")]
use std::sync::Arc;

pub use resyn_core::datamodels::community::{
    CommunityAssignment, CommunityStatus, CommunitySummary,
};

/// Check whether community detection has been run for the current corpus.
#[server(GetCommunityStatus, "/api")]
pub async fn get_community_status() -> Result<CommunityStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        resyn_core::graph_analytics::community::load_community_status(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return summaries for all communities (top papers, keywords, shared methods).
///
/// Assembles summaries on-demand from cached assignment rows — no compute triggered.
#[server(GetAllCommunitySummaries, "/api")]
pub async fn get_all_community_summaries() -> Result<Vec<CommunitySummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        resyn_core::graph_analytics::community::compute_community_summaries(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return the summary for a single community by community_id.
///
/// Returns `None` when no community with that id exists in the current corpus.
#[server(GetCommunitySummary, "/api")]
pub async fn get_community_summary(
    community_id: u32,
) -> Result<Option<CommunitySummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let all = resyn_core::graph_analytics::community::compute_community_summaries(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(all.into_iter().find(|s| s.community_id == community_id))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return the community_id assigned to a specific paper, or None if not yet assigned.
#[server(GetCommunityForPaper, "/api")]
pub async fn get_community_for_paper(arxiv_id: String) -> Result<Option<u32>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::CommunityRepository;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let repo = CommunityRepository::new(&db);
        Ok(repo
            .get_by_paper(&arxiv_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .map(|a| a.community_id))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return all community assignments for the current corpus fingerprint.
///
/// Used by the graph renderer (Plan 02) to color nodes by community.
/// Returns an empty vec when no assignments exist for the current corpus.
#[server(GetCommunityAssignments, "/api")]
pub async fn get_community_assignments() -> Result<Vec<CommunityAssignment>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::CommunityRepository;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let status = resyn_core::graph_analytics::community::load_community_status(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let fp = match status.fingerprint {
            Some(fp) => fp,
            None => return Ok(vec![]),
        };
        let repo = CommunityRepository::new(&db);
        repo.get_all_for_fingerprint(&fp)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
