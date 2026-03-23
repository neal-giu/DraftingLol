#[derive(Debug, Clone, Default)]
pub struct RecommendationService;

impl RecommendationService {
    #[must_use]
    pub const fn engine_name(&self) -> &'static str {
        "draft_recommendation_engine"
    }
}
