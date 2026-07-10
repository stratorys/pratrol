use std::sync::Arc;

use moka::future::Cache;
use tracing::info;

use super::constants::{
    DEDUP_CACHE_CAPACITY,
    DEDUP_TTL,
};
use super::{
    assess,
    decide,
    load,
};
use crate::agent::harness::Harness;
use crate::domains::github::traits::GitHubApp;
use crate::domains::triage::entity::TriageRequest;
use crate::domains::triage::error::TriageError;

type DedupKey = (u64, String, String, u64, String);

pub struct Triage {
    github: Arc<dyn GitHubApp>,
    harness: Harness,
    inflight: Cache<DedupKey, ()>,
}

impl Triage {
    pub fn new(
        github: Arc<dyn GitHubApp>,
        harness: Harness,
    ) -> Self {
        let inflight = Cache::builder()
            .max_capacity(DEDUP_CACHE_CAPACITY)
            .time_to_live(DEDUP_TTL)
            .build();

        Self {
            github,
            harness,
            inflight,
        }
    }

    pub async fn execute(
        &self,
        request: TriageRequest,
    ) -> Result<(), TriageError> {
        let key = (
            request.installation_id,
            request.owner.clone(),
            request.repo.clone(),
            request.pr_number,
            request.head_sha.clone(),
        );
        let entry = self.inflight.entry(key).or_insert_with(async {}).await;
        if !entry.is_fresh() {
            info!(
                message = "Skipping PR, duplicate delivery already in flight.",
                triage_id = %request.id,
                pr_number = request.pr_number,
            );
            return Ok(());
        }

        let Some(context) = load::context(self.github.as_ref(), request).await? else {
            return Ok(());
        };

        let assessment = assess::pull_request(&self.harness, &context).await;
        let decision = decide::triage(&context, assessment);
        let publication = decide::review(&context, &decision)?;

        crate::app::review::publish::review(context.client.as_ref(), publication).await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cache,
        DEDUP_TTL,
        DedupKey,
    };

    fn claim_cache() -> Cache<DedupKey, ()> { Cache::builder().time_to_live(DEDUP_TTL).build() }

    #[tokio::test]
    async fn test_dedup_first_claim_is_fresh_duplicate_is_not() {
        let cache = claim_cache();
        let key: DedupKey = (
            1,
            "owner".to_owned(),
            "repo".to_owned(),
            7,
            "sha-a".to_owned(),
        );

        let first = cache.entry(key.clone()).or_insert_with(async {}).await;
        assert!(first.is_fresh(), "first delivery should win the claim");

        let duplicate = cache.entry(key).or_insert_with(async {}).await;
        assert!(
            !duplicate.is_fresh(),
            "duplicate delivery of the same head sha should be skipped"
        );
    }

    #[tokio::test]
    async fn test_dedup_new_head_sha_is_not_skipped() {
        let cache = claim_cache();
        let first: DedupKey = (
            1,
            "owner".to_owned(),
            "repo".to_owned(),
            7,
            "sha-a".to_owned(),
        );
        let updated: DedupKey = (
            1,
            "owner".to_owned(),
            "repo".to_owned(),
            7,
            "sha-b".to_owned(),
        );

        assert!(
            cache.entry(first).or_insert_with(async {}).await.is_fresh(),
            "first head sha should win the claim"
        );
        assert!(
            cache
                .entry(updated)
                .or_insert_with(async {})
                .await
                .is_fresh(),
            "a new head sha on the same PR should not be treated as a duplicate"
        );
    }
}
