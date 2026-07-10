use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    pub action: PullRequestAction,
    pub installation: Installation,
    pub repository: Repository,
    pub pull_request: PullRequestPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestAction {
    Opened,
    ReadyForReview,
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Installation {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestPayload {
    pub number: u64,
    pub title: String,
    pub user: PullRequestUser,
    pub head: PullRequestHead,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestUser {
    pub login: String,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestHead {
    pub sha: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_action(value: &str) -> PullRequestAction {
        serde_json::from_value(serde_json::json!(value)).expect("action should deserialize")
    }

    #[test]
    fn test_known_actions_deserialize() {
        assert!(matches!(parse_action("opened"), PullRequestAction::Opened));
        assert!(matches!(
            parse_action("ready_for_review"),
            PullRequestAction::ReadyForReview
        ));
    }

    #[test]
    fn test_unknown_action_falls_back_to_other() {
        assert!(matches!(
            parse_action("synchronize"),
            PullRequestAction::Other
        ));
        assert!(matches!(parse_action("closed"), PullRequestAction::Other));
    }

    #[test]
    fn test_event_with_head_sha_deserializes() {
        let payload = serde_json::json!({
            "action": "opened",
            "installation": { "id": 42 },
            "repository": { "full_name": "owner/repo" },
            "pull_request": {
                "number": 7,
                "title": "Add feature",
                "user": { "login": "octocat" },
                "head": { "sha": "abc123" }
            }
        });
        let event: WebhookEvent =
            serde_json::from_value(payload).expect("event should deserialize");
        assert_eq!(event.pull_request.head.sha, "abc123");
        assert!(matches!(event.action, PullRequestAction::Opened));
        assert!(!event.pull_request.draft, "draft should default to false");
    }
}
