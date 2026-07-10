use std::sync::Arc;

use tracing::{
    info,
    warn,
};

use crate::domains::github::entity::CommitInfo;
use crate::domains::github::error::GitHubError;
use crate::domains::github::traits::{
    ContributorReader,
    GitHubApp,
    GitHubClient,
    HistoryReader,
    PullRequestReader,
};
use crate::domains::history::entity::HistorySignals;
use crate::domains::history::evaluate;
use crate::domains::scoring::entity::ProfileSignals;
use crate::domains::triage::entity::TriageRequest;
use crate::domains::triage::error::TriageError;

pub struct Context {
    pub request: TriageRequest,
    pub client: Arc<dyn GitHubClient>,
    pub profile: ProfileSignals,
    pub diff: String,
    pub commits: Vec<CommitInfo>,
    pub history: Option<HistorySignals>,
}

pub async fn context(
    github: &dyn GitHubApp,
    request: TriageRequest,
) -> Result<Option<Context>, TriageError> {
    let client = github.installation_client(request.installation_id).await?;

    if PullRequestReader::has_pratrol_review(
        &*client,
        &request.owner,
        &request.repo,
        request.pr_number,
    )
    .await?
    {
        info!(
            message = "Skipping PR, already triaged.",
            triage_id = %request.id,
            pr_number = request.pr_number,
        );
        return Ok(None);
    }

    let login = &request.author_login;
    let owner = &request.owner;
    let repo = &request.repo;
    let (user, events_count, orgs_count, merged_target, merged_global, diff, commits) = tokio::try_join!(
        ContributorReader::fetch_user(&*client, login),
        ContributorReader::fetch_events_count(&*client, login),
        ContributorReader::fetch_orgs_count(&*client, login),
        ContributorReader::fetch_merged_prs(&*client, login, owner, repo),
        ContributorReader::fetch_merged_prs_global(&*client, login),
        PullRequestReader::fetch_diff(&*client, owner, repo, request.pr_number),
        PullRequestReader::fetch_commits(&*client, owner, repo, request.pr_number),
    )?;

    let history = match history(client.as_ref(), login, owner, repo, &request.title).await {
        Ok(signals) => Some(signals),
        Err(error) => {
            warn!(
                message = "History search failed, skipping history signals.",
                %error,
                triage_id = %request.id,
                pr_number = request.pr_number,
            );
            None
        }
    };

    Ok(Some(Context {
        request,
        client,
        profile: ProfileSignals {
            account_age_days: user.account_age_days,
            public_repos: user.public_repos,
            followers: user.followers,
            public_contributions: events_count,
            prs_merged_target_repo: merged_target,
            prs_merged_elsewhere: merged_global,
            org_memberships: orgs_count,
        },
        diff,
        commits,
        history,
    }))
}

async fn history(
    client: &dyn GitHubClient,
    login: &str,
    owner: &str,
    repo: &str,
    title: &str,
) -> Result<HistorySignals, GitHubError> {
    let author_repo = HistoryReader::search_rejected_prs_by_author(client, login, owner, repo);
    let global = HistoryReader::search_rejected_prs_by_author_global(client, login);

    match evaluate::title_keywords(title) {
        Some(keywords) => {
            let title_result =
                HistoryReader::search_rejected_prs_by_title(client, &keywords, owner, repo);
            tokio::try_join!(author_repo, title_result, global).map(
                |(author_repo, title_result, author_global)| HistorySignals {
                    author_in_repo: author_repo.total_count,
                    author_in_repo_items: author_repo.items,
                    title_in_repo: title_result.total_count,
                    title_in_repo_items: title_result.items,
                    author_global,
                },
            )
        }
        None => tokio::try_join!(author_repo, global).map(|(author_repo, author_global)| {
            HistorySignals {
                author_in_repo: author_repo.total_count,
                author_in_repo_items: author_repo.items,
                title_in_repo: 0,
                title_in_repo_items: vec![],
                author_global,
            }
        }),
    }
}
