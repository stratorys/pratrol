use std::fmt;

pub struct Score {
    pub value: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Tier {
    High,
    Medium,
    Low,
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tier::High => write!(f, "High"),
            Tier::Medium => write!(f, "Medium"),
            Tier::Low => write!(f, "Low"),
        }
    }
}

impl Tier {
    pub fn icon(&self) -> &'static str {
        match self {
            Tier::High => "+",
            Tier::Medium => "~",
            Tier::Low => "-",
        }
    }
}

pub struct ProfileSignals {
    pub account_age_days: u32,
    pub public_repos: u32,
    pub followers: u32,
    pub public_contributions: u32,
    pub prs_merged_target_repo: u32,
    pub prs_merged_elsewhere: u32,
    pub org_memberships: u32,
}

pub struct QualitySignals {
    pub code_coherence: f64,
    pub commit_quality: f64,
    pub risk_level: f64,
    pub suspicious_patterns: f64,
}
