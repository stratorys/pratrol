use tracing::warn;

use crate::domains::github::traits::{
    GitHubClient,
    LabelManager,
};
use crate::domains::scoring::entity::Tier;

const REPEAT_OFFENDER_LABEL: &str = "patrol:repeat-offender";
const REPEAT_OFFENDER_COLOR: &str = "e4a012";
const REPEAT_OFFENDER_DESCRIPTION: &str = "Author has multiple closed-without-merge PRs";

pub async fn apply<C: GitHubClient + ?Sized>(
    client: &C,
    owner: &str,
    repo: &str,
    pr_number: u64,
    tier: Tier,
    repeat_offender: bool,
    triage_id: impl std::fmt::Display,
) {
    let labels = std::iter::once(tier.label().to_owned())
        .chain(repeat_offender.then(|| REPEAT_OFFENDER_LABEL.to_owned()))
        .collect::<Vec<_>>();

    match LabelManager::ensure_label(
        client,
        owner,
        repo,
        tier.label().to_owned(),
        tier.label_color().to_owned(),
        tier.label_description().to_owned(),
    )
    .await
    {
        Ok(()) => {
            let repeat_label_result = if repeat_offender {
                Some(
                    LabelManager::ensure_label(
                        client,
                        owner,
                        repo,
                        REPEAT_OFFENDER_LABEL.to_owned(),
                        REPEAT_OFFENDER_COLOR.to_owned(),
                        REPEAT_OFFENDER_DESCRIPTION.to_owned(),
                    )
                    .await,
                )
            } else {
                None
            };

            if let Some(Err(error)) = repeat_label_result {
                warn!(
                    message = "Failed to ensure repeat-offender label exists.",
                    triage_id = %triage_id,
                    pr_number,
                    %error,
                );
            }

            if let Err(error) =
                LabelManager::add_labels(client, owner, repo, pr_number, labels).await
            {
                warn!(
                    message = "Failed to add labels to PR.",
                    triage_id = %triage_id,
                    pr_number,
                    %error,
                );
            }
        }
        Err(error) => {
            warn!(
                message = "Failed to ensure label exists.",
                triage_id = %triage_id,
                pr_number,
                label = tier.label(),
                %error,
            );
        }
    }
}
