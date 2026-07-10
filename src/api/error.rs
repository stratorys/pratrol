use axum::http::StatusCode;
use axum::response::{
    IntoResponse,
    Response,
};
use tracing::error;

use crate::domains::triage::error::TriageError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("invalid webhook signature")]
    InvalidSignature,

    #[error("invalid webhook payload")]
    InvalidPayload,

    #[error("triage failed")]
    Triage(#[from] TriageError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!(message = "Request failed.", error = %self);

        let status = match &self {
            ApiError::InvalidSignature => StatusCode::UNAUTHORIZED,
            ApiError::InvalidPayload => StatusCode::BAD_REQUEST,
            ApiError::Triage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
