use crate::application::{
    recommendations::{recommend_candidates, RecommendationRequest, RecommendationResponse},
    review::{review_draft, DraftReviewRequest, DraftReviewResponse},
};

#[tauri::command]
pub fn recommend_draft_candidates(request: RecommendationRequest) -> RecommendationResponse {
    recommend_candidates(request)
}

#[tauri::command]
pub fn review_completed_draft(request: DraftReviewRequest) -> DraftReviewResponse {
    review_draft(request)
}
