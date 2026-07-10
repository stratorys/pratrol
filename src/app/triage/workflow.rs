use std::sync::Arc;

use super::{
    assess,
    decide,
    load,
};
use crate::agent::harness::Harness;
use crate::domains::github::traits::GitHubApp;
use crate::domains::triage::entity::TriageRequest;
use crate::domains::triage::error::TriageError;

pub struct Triage {
    github: Arc<dyn GitHubApp>,
    harness: Harness,
}

impl Triage {
    pub fn new(
        github: Arc<dyn GitHubApp>,
        harness: Harness,
    ) -> Self {
        Self {
            github,
            harness,
        }
    }

    pub async fn execute(
        &self,
        request: TriageRequest,
    ) -> Result<(), TriageError> {
        let Some(context) = load::context(self.github.as_ref(), request).await? else {
            return Ok(());
        };

        let assessment = assess::pull_request(&self.harness, &context).await;
        let decision = decide::triage(&context, assessment);
        let publication = decide::review(&context, &decision)?;

        crate::app::review::publish::review(context.client.as_ref(), publication).await
    }
}
