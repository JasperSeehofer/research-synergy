use leptos::prelude::*;
use resyn_core::analysis::aggregation::MethodMatrix;

/// Return the method-category co-occurrence matrix across all annotations.
///
/// The matrix-building logic is extracted to `resyn_core::analysis::aggregation::build_method_matrix`
/// so it can be unit-tested without any Leptos or DB dependencies.
#[server(GetMethodMatrix, "/api")]
pub async fn get_method_matrix() -> Result<MethodMatrix, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::analysis::aggregation::build_method_matrix;
        use resyn_core::database::queries::LlmAnnotationRepository;
        use resyn_core::datamodels::llm_annotation::LlmAnnotation;

        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let annotations: Vec<LlmAnnotation> = LlmAnnotationRepository::new(&db)
            .get_all_annotations()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(build_method_matrix(&annotations))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return a drilldown method matrix for a specific category pair.
///
/// Filters annotations to those that have methods in both cat_a and cat_b,
/// then returns the matrix for those annotations only.
#[server(GetMethodDrilldown, "/api")]
pub async fn get_method_drilldown(cat_a: String, cat_b: String) -> Result<MethodMatrix, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::analysis::aggregation::build_method_matrix;
        use resyn_core::database::queries::LlmAnnotationRepository;
        use resyn_core::datamodels::llm_annotation::LlmAnnotation;

        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let annotations: Vec<LlmAnnotation> = LlmAnnotationRepository::new(&db)
            .get_all_annotations()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Filter to annotations that have methods in both selected categories.
        let filtered: Vec<LlmAnnotation> = annotations
            .into_iter()
            .filter(|ann| {
                let cats: Vec<&str> = ann.methods.iter().map(|m| m.category.as_str()).collect();
                cats.contains(&cat_a.as_str()) && cats.contains(&cat_b.as_str())
            })
            .collect();

        Ok(build_method_matrix(&filtered))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
