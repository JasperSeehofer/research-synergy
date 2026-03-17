use leptos::prelude::*;
use resyn_core::analysis::aggregation::RankedProblem;

/// Return open problems ranked by recurrence frequency across all annotations.
///
/// The aggregation logic is extracted to `resyn_core::analysis::aggregation::aggregate_open_problems`
/// so it can be unit-tested without any Leptos or DB dependencies.
#[server(GetOpenProblemsRanked, "/api")]
pub async fn get_open_problems_ranked() -> Result<Vec<RankedProblem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::analysis::aggregation::aggregate_open_problems;
        use resyn_core::database::queries::LlmAnnotationRepository;
        use resyn_core::datamodels::llm_annotation::LlmAnnotation;

        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let annotations: Vec<LlmAnnotation> = LlmAnnotationRepository::new(&db)
            .get_all_annotations()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(aggregate_open_problems(&annotations))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
