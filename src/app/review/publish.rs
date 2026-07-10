use tracing::info;

use crate::app::labels::apply;
use crate::domains::github::traits::{
    GitHubClient,
    ReviewPublisher,
};
use crate::domains::scoring::entity::Tier;
use crate::domains::triage::entity::TriageId;
use crate::domains::triage::error::TriageError;

pub struct Publication {
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
    pub head_sha: String,
    pub markdown: String,
    pub tier: Tier,
    pub repeat_offender: bool,
    pub history_penalty: f64,
    pub triage_id: TriageId,
}

pub async fn review(
    client: &dyn GitHubClient,
    publication: Publication,
) -> Result<(), TriageError> {
    ReviewPublisher::post_review(
        client,
        &publication.owner,
        &publication.repo,
        publication.pr_number,
        &publication.head_sha,
        &publication.markdown,
    )
    .await?;

    apply::apply(
        client,
        &publication.owner,
        &publication.repo,
        publication.pr_number,
        publication.tier,
        publication.repeat_offender,
        publication.triage_id,
    )
    .await;

    info!(
        message = "Posted triage review.",
        triage_id = %publication.triage_id,
        pr_number = publication.pr_number,
        tier = %publication.tier,
        history_penalty = publication.history_penalty,
        repeat_offender = publication.repeat_offender,
    );

    Ok(())
}
