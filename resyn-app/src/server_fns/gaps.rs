use leptos::prelude::*;
use resyn_core::datamodels::gap_finding::GapFinding;

/// Return all gap findings from the database.
#[server(GetGapFindings, "/api")]
pub async fn get_gap_findings() -> Result<Vec<GapFinding>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::GapFindingRepository;
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let findings = GapFindingRepository::new(&db)
            .get_all_gap_findings()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(findings)
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
