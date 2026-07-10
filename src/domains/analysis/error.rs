#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("analysis response is not valid json")]
    Json,

    #[error("analysis field value out of valid range")]
    FieldOutOfRange,
}
