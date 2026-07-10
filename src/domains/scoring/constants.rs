pub(crate) const ACCOUNT_AGE_DAYS_CAP: f64 = 1095.0;
pub(crate) const PUBLIC_REPOS_CAP: f64 = 50.0;
pub(crate) const FOLLOWERS_CAP: f64 = 100.0;
pub(crate) const CONTRIBUTIONS_CAP: f64 = 300.0;
pub(crate) const PRS_TARGET_CAP: f64 = 10.0;
pub(crate) const PRS_ELSEWHERE_CAP: f64 = 50.0;
pub(crate) const ORGS_CAP: f64 = 5.0;

pub(crate) const WEIGHT_ACCOUNT_AGE: f64 = 0.15;
pub(crate) const WEIGHT_PUBLIC_REPOS: f64 = 0.10;
pub(crate) const WEIGHT_FOLLOWERS: f64 = 0.05;
pub(crate) const WEIGHT_CONTRIBUTIONS: f64 = 0.15;
pub(crate) const WEIGHT_PRS_TARGET: f64 = 0.30;
pub(crate) const WEIGHT_PRS_ELSEWHERE: f64 = 0.15;
pub(crate) const WEIGHT_ORGS: f64 = 0.10;

pub(crate) const WEIGHT_CODE_COHERENCE: f64 = 0.30;
pub(crate) const WEIGHT_COMMIT_QUALITY: f64 = 0.20;
pub(crate) const WEIGHT_RISK_LEVEL: f64 = 0.25;
pub(crate) const WEIGHT_SUSPICIOUS_PATTERNS: f64 = 0.25;
pub(crate) const QUALITY_DIMENSION_MAX: f64 = 10.0;

pub(crate) const WEIGHT_PROFILE: f64 = 0.4;
pub(crate) const WEIGHT_QUALITY: f64 = 0.6;

pub(crate) const TIER_HIGH_MIN: f64 = 70.0;
pub(crate) const TIER_MEDIUM_MIN: f64 = 40.0;

pub(crate) const PARTIAL_ANALYSIS_SCORE_CAP: f64 = 39.0;

const _: () = assert!(
    PARTIAL_ANALYSIS_SCORE_CAP < TIER_MEDIUM_MIN,
    "partial-analysis cap must stay below the Medium tier floor so degraded analyses stay Low"
);
