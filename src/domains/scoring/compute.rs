use super::entity::{
    ProfileSignals,
    QualitySignals,
    Score,
    Tier,
};

const ACCOUNT_AGE_DAYS_CAP: f64 = 1095.0;
const PUBLIC_REPOS_CAP: f64 = 50.0;
const FOLLOWERS_CAP: f64 = 100.0;
const CONTRIBUTIONS_CAP: f64 = 300.0;
const PRS_TARGET_CAP: f64 = 10.0;
const PRS_ELSEWHERE_CAP: f64 = 50.0;
const ORGS_CAP: f64 = 5.0;

const WEIGHT_ACCOUNT_AGE: f64 = 0.15;
const WEIGHT_PUBLIC_REPOS: f64 = 0.10;
const WEIGHT_FOLLOWERS: f64 = 0.05;
const WEIGHT_CONTRIBUTIONS: f64 = 0.15;
const WEIGHT_PRS_TARGET: f64 = 0.30;
const WEIGHT_PRS_ELSEWHERE: f64 = 0.15;
const WEIGHT_ORGS: f64 = 0.10;

const WEIGHT_CODE_COHERENCE: f64 = 0.30;
const WEIGHT_COMMIT_QUALITY: f64 = 0.20;
const WEIGHT_RISK_LEVEL: f64 = 0.25;
const WEIGHT_SUSPICIOUS_PATTERNS: f64 = 0.25;
const QUALITY_DIMENSION_MAX: f64 = 10.0;

const WEIGHT_PROFILE: f64 = 0.4;
const WEIGHT_QUALITY: f64 = 0.6;

const TIER_HIGH_MIN: f64 = 70.0;
const TIER_MEDIUM_MIN: f64 = 40.0;

pub fn profile_score(signals: &ProfileSignals) -> Score {
    let account_age_norm = (f64::from(signals.account_age_days) / ACCOUNT_AGE_DAYS_CAP).min(1.0);
    let public_repos_norm = (f64::from(signals.public_repos) / PUBLIC_REPOS_CAP).min(1.0);
    let followers_norm = (f64::from(signals.followers) / FOLLOWERS_CAP).min(1.0);
    let contributions_norm = (f64::from(signals.public_contributions) / CONTRIBUTIONS_CAP).min(1.0);
    let prs_target_norm = (f64::from(signals.prs_merged_target_repo) / PRS_TARGET_CAP).min(1.0);
    let prs_elsewhere_norm = (f64::from(signals.prs_merged_elsewhere) / PRS_ELSEWHERE_CAP).min(1.0);
    let orgs_norm = (f64::from(signals.org_memberships) / ORGS_CAP).min(1.0);

    let value = (account_age_norm * WEIGHT_ACCOUNT_AGE
        + public_repos_norm * WEIGHT_PUBLIC_REPOS
        + followers_norm * WEIGHT_FOLLOWERS
        + contributions_norm * WEIGHT_CONTRIBUTIONS
        + prs_target_norm * WEIGHT_PRS_TARGET
        + prs_elsewhere_norm * WEIGHT_PRS_ELSEWHERE
        + orgs_norm * WEIGHT_ORGS)
        * 100.0;

    Score {
        value,
    }
}

pub fn quality_score(signals: &QualitySignals) -> Score {
    let value = ((signals.code_coherence / QUALITY_DIMENSION_MAX) * WEIGHT_CODE_COHERENCE
        + (signals.commit_quality / QUALITY_DIMENSION_MAX) * WEIGHT_COMMIT_QUALITY
        + ((QUALITY_DIMENSION_MAX - signals.risk_level) / QUALITY_DIMENSION_MAX)
            * WEIGHT_RISK_LEVEL
        + ((QUALITY_DIMENSION_MAX - signals.suspicious_patterns) / QUALITY_DIMENSION_MAX)
            * WEIGHT_SUSPICIOUS_PATTERNS)
        * 100.0;

    Score {
        value,
    }
}

pub fn combine(
    profile_score: f64,
    quality_score: f64,
) -> (f64, Tier) {
    let combined = WEIGHT_PROFILE * profile_score + WEIGHT_QUALITY * quality_score;
    let tier = tier_from_score(combined);
    (combined, tier)
}

pub fn tier_from_score(score: f64) -> Tier {
    if score >= TIER_HIGH_MIN {
        Tier::High
    } else if score >= TIER_MEDIUM_MIN {
        Tier::Medium
    } else {
        Tier::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_profile_score_all_zero() {
        let signals = ProfileSignals {
            account_age_days: 0,
            public_repos: 0,
            followers: 0,
            public_contributions: 0,
            prs_merged_target_repo: 0,
            prs_merged_elsewhere: 0,
            org_memberships: 0,
        };
        let score = profile_score(&signals);
        assert!(
            score.value.abs() < f64::EPSILON,
            "expected 0, got {}",
            score.value
        );
    }

    #[test]
    fn test_compute_profile_score_all_max() {
        let signals = ProfileSignals {
            account_age_days: 2000,
            public_repos: 100,
            followers: 200,
            public_contributions: 500,
            prs_merged_target_repo: 20,
            prs_merged_elsewhere: 100,
            org_memberships: 10,
        };
        let score = profile_score(&signals);
        assert!(
            (score.value - 100.0).abs() < f64::EPSILON,
            "expected 100, got {}",
            score.value
        );
    }

    #[test]
    fn test_compute_quality_score_perfect() {
        let signals = QualitySignals {
            code_coherence: 10.0,
            commit_quality: 10.0,
            risk_level: 0.0,
            suspicious_patterns: 0.0,
        };
        let score = quality_score(&signals);
        assert!(
            (score.value - 100.0).abs() < f64::EPSILON,
            "expected 100, got {}",
            score.value
        );
    }

    #[test]
    fn test_compute_quality_score_worst() {
        let signals = QualitySignals {
            code_coherence: 0.0,
            commit_quality: 0.0,
            risk_level: 10.0,
            suspicious_patterns: 10.0,
        };
        let score = quality_score(&signals);
        assert!(
            score.value.abs() < f64::EPSILON,
            "expected 0, got {}",
            score.value
        );
    }

    #[test]
    fn test_combine_weights() {
        let (combined, _tier) = combine(100.0, 0.0);
        assert!(
            (combined - 40.0).abs() < f64::EPSILON,
            "expected 40, got {combined}"
        );

        let (combined, _tier) = combine(0.0, 100.0);
        assert!(
            (combined - 60.0).abs() < f64::EPSILON,
            "expected 60, got {combined}"
        );
    }

    #[test]
    fn test_tier_boundaries() {
        assert!(
            matches!(tier_from_score(70.0), Tier::High),
            "70 should be High"
        );
        assert!(
            matches!(tier_from_score(40.0), Tier::Medium),
            "40 should be Medium"
        );
        assert!(
            matches!(tier_from_score(39.99), Tier::Low),
            "39.99 should be Low"
        );
    }
}
