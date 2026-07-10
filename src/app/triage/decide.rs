use super::assess::Assessment;
use super::load::Context;
use crate::app::review::publish::Publication;
use crate::domains::comment::entity::CommentPayload;
use crate::domains::comment::render;
use crate::domains::history::entity::HistoryResult;
use crate::domains::history::evaluate;
use crate::domains::scoring::compute;
use crate::domains::scoring::constants::PARTIAL_ANALYSIS_SCORE_CAP;
use crate::domains::scoring::entity::{
    Score,
    Tier,
};
use crate::domains::triage::error::TriageError;

pub struct Decision {
    pub adjusted_score: f64,
    pub profile_score: f64,
    pub combined_tier: Tier,
    pub profile_tier: Tier,
    pub quality_tier: Tier,
    pub history: HistoryResult,
    pub partial: bool,
    pub quality_score: Score,
    pub summary: String,
    pub key_signal: String,
    pub recommendation: String,
}

pub fn triage(
    context: &Context,
    assessment: Assessment,
) -> Decision {
    let profile_score = compute::profile_score(&context.profile);
    let history = match &context.history {
        Some(signals) => evaluate::history(signals, context.request.pr_number),
        None => evaluate::unavailable(),
    };
    let (raw_score, _) = compute::combine(profile_score.value(), assessment.quality_score.value());

    let adjusted_score = (raw_score - history.penalty).max(0.0);
    let adjusted_score = if assessment.partial {
        adjusted_score.min(PARTIAL_ANALYSIS_SCORE_CAP)
    } else {
        adjusted_score
    };

    Decision {
        adjusted_score,
        profile_score: profile_score.value(),
        combined_tier: compute::tier_from_score(adjusted_score),
        profile_tier: compute::tier_from_score(profile_score.value()),
        quality_tier: compute::tier_from_score(assessment.quality_score.value()),
        history,
        partial: assessment.partial,
        quality_score: assessment.quality_score,
        summary: assessment.summary,
        key_signal: assessment.key_signal,
        recommendation: assessment.recommendation,
    }
}

pub fn review(
    context: &Context,
    decision: &Decision,
) -> Result<Publication, TriageError> {
    let payload = CommentPayload {
        profile_score: decision.profile_score,
        profile_tier_label: decision.profile_tier.to_string(),
        profile_tier_icon: decision.profile_tier.icon().to_owned(),
        quality_score: decision.quality_score.value(),
        quality_tier_label: decision.quality_tier.to_string(),
        quality_tier_icon: decision.quality_tier.icon().to_owned(),
        combined_score: decision.adjusted_score,
        combined_tier_label: decision.combined_tier.to_string(),
        combined_tier_icon: decision.combined_tier.icon().to_owned(),
        summary: decision.summary.clone(),
        key_signal: decision.key_signal.clone(),
        recommendation: decision.recommendation.clone(),
        analysis_partial: decision.partial,
        history: decision.history.presentation.clone(),
    };

    Ok(Publication {
        owner: context.request.owner.clone(),
        repo: context.request.repo.clone(),
        pr_number: context.request.pr_number,
        head_sha: context.request.head_sha.clone(),
        markdown: render::markdown(&payload)?,
        tier: decision.combined_tier,
        repeat_offender: decision.history.is_repeat_offender,
        history_penalty: decision.history.penalty,
        triage_id: context.request.id,
    })
}
