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
use crate::domains::reviewer::service::ReviewerService;
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
    reviewer: ReviewerService,
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
            reviewer: ReviewerService::new(),
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

        let prompt = self.analysis.build_prompt(&diff, &commits);

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

        client
            .post_review(owner, repo, pr_number, &markdown)
            .await?;

        info!(
            message = "Posted triage review.",
            triage_id = %triage_id,
            pr_number,
            combined_score,
            tier = %combined_tier,
        );

        match self
            .reviewer
            .assign_reviewers(&client, owner, repo, pr_number, login)
            .await
        {
            Ok(()) => {}
            Err(error) => {
                warn!(
                    message = "Failed to assign reviewers.",
                    %error,
                    triage_id = %triage_id,
                    pr_number,
                );
            }
        }

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

    use async_trait::async_trait;

    use super::*;
    use crate::domains::triage::entity::TriageId;
    use crate::ports::github::{
        GitHubApp,
        GitHubClient,
        GitHubError,
        UserInfo,
    };
    use crate::ports::mistral::{
        MistralError,
        MistralPort,
    };

    struct MockGitHubApp {
        should_fail: bool,
    }

    struct MockGitHubClient;

    #[async_trait]
    impl GitHubApp for MockGitHubApp {
        type Client = MockGitHubClient;

        async fn installation_client(
            &self,
            _installation_id: u64,
        ) -> Result<MockGitHubClient, GitHubError> {
            if self.should_fail {
                return Err(GitHubError::UnexpectedStatus);
            }
            Ok(MockGitHubClient)
        }
    }

    #[async_trait]
    impl GitHubClient for MockGitHubClient {
        async fn fetch_user(
            &self,
            _login: &str,
        ) -> Result<UserInfo, GitHubError> {
            Ok(UserInfo {
                account_age_days: 365,
                public_repos: 10,
                followers: 5,
            })
        }

        async fn fetch_events_count(
            &self,
            _login: &str,
        ) -> Result<u32, GitHubError> {
            Ok(50)
        }

        async fn fetch_orgs_count(
            &self,
            _login: &str,
        ) -> Result<u32, GitHubError> {
            Ok(2)
        }

        async fn fetch_merged_prs(
            &self,
            _login: &str,
            _owner: &str,
            _repo: &str,
        ) -> Result<u32, GitHubError> {
            Ok(3)
        }

        async fn fetch_merged_prs_global(
            &self,
            _login: &str,
        ) -> Result<u32, GitHubError> {
            Ok(15)
        }

        async fn fetch_diff(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
        ) -> Result<String, GitHubError> {
            Ok("diff content".to_owned())
        }

        async fn fetch_commits(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
        ) -> Result<Vec<String>, GitHubError> {
            Ok(vec!["Initial commit".to_owned()])
        }

        async fn post_review(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
            _body: &str,
        ) -> Result<(), GitHubError> {
            Ok(())
        }

        async fn fetch_codeowners(
            &self,
            _owner: &str,
            _repo: &str,
        ) -> Result<Option<String>, GitHubError> {
            Ok(None)
        }

        async fn fetch_pr_files(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
        ) -> Result<Vec<String>, GitHubError> {
            Ok(Vec::new())
        }

        async fn fetch_file_contributors(
            &self,
            _owner: &str,
            _repo: &str,
            _path: &str,
            _limit: u32,
        ) -> Result<Vec<String>, GitHubError> {
            Ok(Vec::new())
        }

        async fn request_reviewers(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
            _users: &[String],
            _teams: &[String],
        ) -> Result<(), GitHubError> {
            Ok(())
        }
    }

    struct MockMistral {
        should_fail: bool,
    }

    #[async_trait]
    impl MistralPort for MockMistral {
        async fn chat_completion(
            &self,
            _prompt: &str,
        ) -> Result<String, MistralError> {
            if self.should_fail {
                return Err(MistralError::EmptyResponse);
            }
            Ok(r#"{"code_coherence": 8.0, "commit_quality": 7.0, "risk_level": 2.0, "suspicious_patterns": 1.0, "summary": "A good PR.", "key_signal": "Clean code.", "recommendation": "Approve."}"#.to_owned())
        }
    }

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

    #[tokio::test]
    async fn test_execute_success() {
        let github = Arc::new(MockGitHubApp {
            should_fail: false,
        });
        let mistral = Arc::new(MockMistral {
            should_fail: false,
        });
        let service = TriageService::new(github, mistral);

        let result = service.execute(sample_request()).await;
        assert!(result.is_ok(), "execute should succeed");
    }

    #[tokio::test]
    async fn test_execute_mistral_failure_uses_fallback() {
        let github = Arc::new(MockGitHubApp {
            should_fail: false,
        });
        let mistral = Arc::new(MockMistral {
            should_fail: true,
        });
        let service = TriageService::new(github, mistral);

        let result = service.execute(sample_request()).await;
        assert!(
            result.is_ok(),
            "execute should succeed with fallback when Mistral fails"
        );
    }

    #[tokio::test]
    async fn test_execute_github_failure_propagates() {
        let github = Arc::new(MockGitHubApp {
            should_fail: true,
        });
        let mistral = Arc::new(MockMistral {
            should_fail: false,
        });
        let service = TriageService::new(github, mistral);

        let result = service.execute(sample_request()).await;
        assert!(result.is_err(), "execute should fail when GitHub fails");
        assert!(
            matches!(result.err(), Some(TriageError::GitHub(_))),
            "should be GitHub error"
        );
    }
}
