#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("analysis field value out of valid range")]
    FieldOutOfRange,
}
