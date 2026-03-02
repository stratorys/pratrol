use std::sync::Arc;

use tracing::{
    info,
    warn,
};

use super::entity::TriageRequest;
use super::error::TriageError;
use crate::domains::analysis::service::AnalysisService;
use crate::domains::comment::entity::CommentPayload;
use crate::domains::comment::service::CommentService;
use crate::domains::scoring::entity::{
    ProfileSignals,
    QualitySignals,
    Score,
};
use crate::domains::scoring::service::ScoringService;
use crate::ports::github::{
    GitHubApp,
    GitHubClient,
};
use crate::ports::mistral::MistralPort;

pub struct TriageService<G, M> {
    github: Arc<G>,
    mistral: Arc<M>,
    scoring: ScoringService,
    analysis: AnalysisService,
    comment: CommentService,
}

impl<G: GitHubApp, M: MistralPort> TriageService<G, M> {
    pub fn new(
        github: Arc<G>,
        mistral: Arc<M>,
    ) -> Self {
        Self {
            github,
            mistral,
            scoring: ScoringService::new(),
            analysis: AnalysisService::new(),
            comment: CommentService::new(),
        }
    }

    pub async fn execute(
        &self,
        request: TriageRequest,
    ) -> Result<(), TriageError> {
        let client = self
            .github
            .installation_client(request.installation_id)
            .await?;

        let triage_id = request.id;
        let login = &request.author_login;
        let owner = &request.owner;
        let repo = &request.repo;
        let pr_number = request.pr_number;

        let (user, events_count, orgs_count, merged_target, merged_global, diff, commits) = tokio::try_join!(
            client.fetch_user(login),
            client.fetch_events_count(login),
            client.fetch_orgs_count(login),
            client.fetch_merged_prs(login, owner, repo),
            client.fetch_merged_prs_global(login),
            client.fetch_diff(owner, repo, pr_number),
            client.fetch_commits(owner, repo, pr_number),
        )?;

        let profile_signals = ProfileSignals {
            account_age_days: user.account_age_days,
            public_repos: user.public_repos,
            followers: user.followers,
            public_contributions: events_count,
            prs_merged_target_repo: merged_target,
            prs_merged_elsewhere: merged_global,
            org_memberships: orgs_count,
        };

        let profile_score = self.scoring.compute_profile_score(&profile_signals);

        let commit_messages: Vec<&str> = commits.iter().map(|ci| ci.message.as_str()).collect();
        let prompt = self.analysis.build_prompt(&diff, &commit_messages);

        let (quality_score, summary, key_signal, recommendation, analysis_partial) =
            match self.mistral.chat_completion(&prompt).await {
                Ok(raw_response) => match self.analysis.parse_response(&raw_response) {
                    Ok(analysis) => {
                        let quality_signals = QualitySignals {
                            code_coherence: analysis.code_coherence,
                            commit_quality: analysis.commit_quality,
                            risk_level: analysis.risk_level,
                            suspicious_patterns: analysis.suspicious_patterns,
                        };
                        let score = self.scoring.compute_quality_score(&quality_signals);
                        (
                            score,
                            analysis.summary,
                            analysis.key_signal,
                            analysis.recommendation,
                            false,
                        )
                    }
                    Err(error) => {
                        warn!(message = "Failed to parse Mistral response.", %error);
                        fallback_analysis()
                    }
                },
                Err(error) => {
                    warn!(message = "Mistral API call failed.", %error);
                    fallback_analysis()
                }
            };

        let (combined_score, combined_tier) = self
            .scoring
            .combine(profile_score.value, quality_score.value);

        let profile_tier = self.scoring.tier_from_score(profile_score.value);
        let quality_tier = self.scoring.tier_from_score(quality_score.value);

        let comment_payload = CommentPayload {
            profile_score: profile_score.value,
            profile_tier_label: profile_tier.to_string(),
            profile_tier_icon: profile_tier.icon().to_owned(),
            quality_score: quality_score.value,
            quality_tier_label: quality_tier.to_string(),
            quality_tier_icon: quality_tier.icon().to_owned(),
            combined_score,
            combined_tier_label: combined_tier.to_string(),
            combined_tier_icon: combined_tier.icon().to_owned(),
            summary,
            key_signal,
            recommendation,
            analysis_partial,
        };

        let markdown = self.comment.render(&comment_payload);

        let head_commit = commits.last().ok_or(TriageError::NoCommits)?;

        client
            .post_review(owner, repo, pr_number, &head_commit.sha, &markdown)
            .await?;

        let label_name = combined_tier.label().to_owned();
        let label_color = combined_tier.label_color().to_owned();
        let label_description = combined_tier.label_description().to_owned();

        match client
            .ensure_label(
                owner,
                repo,
                label_name.clone(),
                label_color,
                label_description,
            )
            .await
        {
            Ok(()) => {
                if let Err(error) = client
                    .add_labels(owner, repo, pr_number, vec![label_name])
                    .await
                {
                    warn!(
                        message = "Failed to add label to PR.",
                        triage_id = %triage_id,
                        pr_number,
                        label = combined_tier.label(),
                        %error,
                    );
                }
            }
            Err(error) => {
                warn!(
                    message = "Failed to ensure label exists.",
                    triage_id = %triage_id,
                    pr_number,
                    label = combined_tier.label(),
                    %error,
                );
            }
        }

        info!(
            message = "Posted triage review.",
            triage_id = %triage_id,
            pr_number,
            combined_score,
            tier = %combined_tier,
        );

        Ok(())
    }
}

fn fallback_analysis() -> (Score, String, String, String, bool) {
    let score = Score {
        value: 50.0,
    };
    let summary = "Analysis was partial due to an error contacting the AI service.".to_owned();
    let key_signal = "AI analysis unavailable.".to_owned();
    let recommendation = "Manual review recommended.".to_owned();
    (score, summary, key_signal, recommendation, true)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::domains::triage::entity::TriageId;
    use crate::ports::github::{
        CommitInfo,
        MockGitHubApp,
        MockGitHubClient,
        UserInfo,
    };
    use crate::ports::mistral::MockMistralPort;

    fn sample_request() -> TriageRequest {
        TriageRequest {
            id: TriageId::new(),
            installation_id: 1,
            owner: "owner".to_owned(),
            repo: "repo".to_owned(),
            pr_number: 42,
            author_login: "user".to_owned(),
        }
    }

    fn setup_successful_client() -> MockGitHubClient {
        let mut client = MockGitHubClient::new();
        client.expect_fetch_user().returning(|_| {
            Ok(UserInfo {
                account_age_days: 365,
                public_repos: 10,
                followers: 5,
            })
        });
        client.expect_fetch_events_count().returning(|_| Ok(50));
        client.expect_fetch_orgs_count().returning(|_| Ok(2));
        client.expect_fetch_merged_prs().returning(|_, _, _| Ok(3));
        client
            .expect_fetch_merged_prs_global()
            .returning(|_| Ok(15));
        client
            .expect_fetch_diff()
            .returning(|_, _, _| Ok("diff content".to_owned()));
        client.expect_fetch_commits().returning(|_, _, _| {
            Ok(vec![CommitInfo {
                sha: "abc123".to_owned(),
                message: "Initial commit".to_owned(),
            }])
        });
        client
            .expect_post_review()
            .returning(|_, _, _, _, _| Ok(()));
        client
            .expect_ensure_label()
            .returning(|_, _, _, _, _| Ok(()));
        client.expect_add_labels().returning(|_, _, _, _| Ok(()));
        client
    }

    fn setup_successful_app() -> MockGitHubApp {
        let mut app = MockGitHubApp::new();
        app.expect_installation_client()
            .returning(|_| Ok(setup_successful_client()));
        app
    }

    fn successful_mistral_response() -> MockMistralPort {
        let mut mistral = MockMistralPort::new();
        mistral.expect_chat_completion().returning(|_| {
            Ok(r#"{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "A good PR.", "key_signal": "Clean code.", "recommendation": "Approve."}"#.to_owned())
        });
        mistral
    }

    #[tokio::test]
    async fn test_execute_success() {
        let github = Arc::new(setup_successful_app());
        let mistral = Arc::new(successful_mistral_response());
        let service = TriageService::new(github, mistral);

        let result = service.execute(sample_request()).await;
        assert!(result.is_ok(), "execute should succeed");
    }

    #[tokio::test]
    async fn test_execute_mistral_failure_uses_fallback() {
        let github = Arc::new(setup_successful_app());
        let mut mistral = MockMistralPort::new();
        mistral
            .expect_chat_completion()
            .returning(|_| Err(crate::ports::mistral::MistralError::EmptyResponse));
        let service = TriageService::new(github, Arc::new(mistral));

        let result = service.execute(sample_request()).await;
        assert!(
            result.is_ok(),
            "execute should succeed with fallback when Mistral fails"
        );
    }

    #[tokio::test]
    async fn test_execute_github_failure_propagates() {
        let mut app = MockGitHubApp::new();
        app.expect_installation_client().returning(|_| {
            let jwt_error =
                jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken);
            Err(crate::ports::github::GitHubError::Jwt(jwt_error))
        });
        let mistral = Arc::new(successful_mistral_response());
        let service = TriageService::new(Arc::new(app), mistral);

        let result = service.execute(sample_request()).await;
        assert!(result.is_err(), "execute should fail when GitHub fails");
        assert!(
            matches!(result.err(), Some(TriageError::GitHub(_))),
            "should be GitHub error"
        );
    }
}
