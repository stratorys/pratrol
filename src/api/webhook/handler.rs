use axum::body::Bytes;
use axum::extract::State;
use axum::http::{
    HeaderMap,
    StatusCode,
};
use axum::response::IntoResponse;
use hmac::{
    Hmac,
    KeyInit,
    Mac,
};
use sha2::Sha256;
use tracing::{
    error,
    info,
};

use crate::AppState;
use crate::api::error::ApiError;
use crate::api::webhook::dto::WebhookEvent;
use crate::domains::triage::entity::{
    TriageId,
    TriageRequest,
};

type HmacSha256 = Hmac<Sha256>;

pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    verify_signature(&state.webhook_secret, &headers, &body)?;

    let event_type = match headers.get("X-GitHub-Event").and_then(|v| v.to_str().ok()) {
        Some(event_type) => event_type,
        None => return Ok(StatusCode::OK),
    };

    if event_type != "pull_request" {
        return Ok(StatusCode::OK);
    }

    let event: WebhookEvent = serde_json::from_slice(&body)?;

    let should_triage = match event.action.as_str() {
        "opened" => !event.pull_request.draft,
        "ready_for_review" => true,
        _ => false,
    };

    if !should_triage {
        return Ok(StatusCode::OK);
    }

    let request = try_into_triage_request(&event).ok_or(ApiError::InvalidPayload)?;

    info!(
        message = "Processing PR.",
        pr_number = request.pr_number,
        repo = %event.repository.full_name,
        author = %request.author_login,
    );

    let triage_id = request.id;
    let pr_number = request.pr_number;
    let repo_full_name = event.repository.full_name.clone();
    let triage_service = state.triage_service.clone();
    tokio::spawn(async move {
        match triage_service.execute(request).await {
            Ok(()) => {}
            Err(error) => {
                error!(
                    message = "Failed to process PR.",
                    %error,
                    triage_id = %triage_id,
                    pr_number,
                    repo = %repo_full_name,
                );
            }
        }
    });

    Ok(StatusCode::OK)
}

fn try_into_triage_request(event: &WebhookEvent) -> Option<TriageRequest> {
    let (owner, repo) = split_full_name(&event.repository.full_name)?;
    Some(TriageRequest {
        id: TriageId::new(),
        installation_id: event.installation.id,
        owner,
        repo,
        pr_number: event.pull_request.number,
        author_login: event.pull_request.user.login.clone(),
        title: event.pull_request.title.clone(),
    })
}

fn verify_signature(
    secret: &str,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), ApiError> {
    let signature_header = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::InvalidSignature)?;

    let hex_signature = signature_header
        .strip_prefix("sha256=")
        .ok_or(ApiError::InvalidSignature)?;

    let signature_bytes = hex::decode(hex_signature).map_err(|_| ApiError::InvalidSignature)?;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| ApiError::InvalidSignature)?;
    mac.update(body);
    mac.verify_slice(&signature_bytes)
        .map_err(|_| ApiError::InvalidSignature)?;

    Ok(())
}

fn split_full_name(full_name: &str) -> Option<(String, String)> {
    let mut parts = full_name.splitn(2, '/');
    let owner = parts.next()?.to_owned();
    let repo = parts.next()?.to_owned();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

#[cfg(test)]
mod tests {
    use hmac::{
        Hmac,
        Mac,
    };
    use sha2::Sha256;

    use super::*;

    fn compute_signature(
        secret: &str,
        body: &[u8],
    ) -> String {
        let mut mac = <Hmac<Sha256>>::new_from_slice(secret.as_bytes())
            .expect("HMAC key should be valid in test");
        mac.update(body);
        let result = mac.finalize().into_bytes();
        format!("sha256={}", hex::encode(result))
    }

    #[test]
    fn test_verify_signature_valid() {
        let secret = "test-secret";
        let body = b"test body";
        let signature = compute_signature(secret, body);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            signature
                .parse()
                .expect("signature should be a valid header value"),
        );

        let result = verify_signature(secret, &headers, body);
        assert!(result.is_ok(), "valid signature should pass");
    }

    #[test]
    fn test_verify_signature_invalid() {
        let secret = "test-secret";
        let body = b"test body";

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Hub-Signature-256",
            "sha256=0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .expect("hex signature should be a valid header value"),
        );

        let result = verify_signature(secret, &headers, body);
        assert!(result.is_err(), "invalid signature should fail");
    }

    #[test]
    fn test_split_full_name_valid() {
        let result = split_full_name("owner/repo");
        assert!(result.is_some(), "should split valid full_name");
        let (owner, repo) = result.expect("split should succeed for owner/repo");
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_split_full_name_invalid() {
        assert!(
            split_full_name("noslash").is_none(),
            "should return None for missing slash"
        );
        assert!(
            split_full_name("/repo").is_none(),
            "should return None for empty owner"
        );
        assert!(
            split_full_name("owner/").is_none(),
            "should return None for empty repo"
        );
    }
}
