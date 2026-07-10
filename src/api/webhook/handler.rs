use axum::extract::State;
use axum::http::{
    HeaderMap,
    StatusCode,
};
use axum::response::IntoResponse;
use tracing::{
    error,
    info,
};

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::api::webhook::dto::{
    PullRequestAction,
    WebhookEvent,
};
use crate::api::webhook::extract::VerifiedBody;
use crate::domains::triage::entity::{
    TriageId,
    TriageRequest,
};

pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    VerifiedBody(body): VerifiedBody,
) -> Result<impl IntoResponse, ApiError> {
    let Some(event_type) = headers.get("X-GitHub-Event").and_then(|v| v.to_str().ok()) else {
        return Ok(StatusCode::OK);
    };

    if event_type != "pull_request" {
        return Ok(StatusCode::OK);
    }

    let event: WebhookEvent = serde_json::from_slice(&body).map_err(|error| {
        error!(message = "Failed to parse webhook payload.", %error);
        ApiError::InvalidPayload
    })?;

    let should_triage = match event.action {
        PullRequestAction::Opened => !event.pull_request.draft,
        PullRequestAction::ReadyForReview => true,
        PullRequestAction::Other => false,
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
    let triage = state.triage.clone();
    let shutdown = state.shutdown.clone();
    state.tasks.spawn(async move {
        tokio::select! {
            () = shutdown.cancelled() => {
                info!(
                    message = "Shutdown requested, abandoning in-flight triage.",
                    triage_id = %triage_id,
                    pr_number,
                    repo = %repo_full_name,
                );
            }
            result = triage.execute(request) => {
                if let Err(error) = result {
                    error!(
                        message = "Failed to process PR.",
                        %error,
                        triage_id = %triage_id,
                        pr_number,
                        repo = %repo_full_name,
                    );
                }
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
        head_sha: event.pull_request.head.sha.clone(),
        author_login: event.pull_request.user.login.clone(),
        title: event.pull_request.title.clone(),
    })
}

fn split_full_name(full_name: &str) -> Option<(String, String)> {
    let (owner, repo) = full_name.split_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_owned(), repo.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

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
