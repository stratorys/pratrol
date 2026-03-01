use axum::Json;
use axum::response::IntoResponse;

pub async fn handle_health() -> impl IntoResponse { Json(serde_json::json!({})) }

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;

    use super::handle_health;

    #[tokio::test]
    async fn health_returns_200_with_empty_json() {
        let app = Router::new().route("/health", get(handle_health));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("Failed to build test request."),
            )
            .await
            .expect("Failed to execute test request.");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("Failed to read response body.")
            .to_bytes();

        assert_eq!(&body[..], b"{}");
    }
}
