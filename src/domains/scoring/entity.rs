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
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Tier::High => write!(f, "High"),
            Tier::Medium => write!(f, "Medium"),
            Tier::Low => write!(f, "Low"),
        }
    }
}

impl Tier {
    pub fn icon(self) -> &'static str {
        match self {
            Tier::High => "+",
            Tier::Medium => "~",
            Tier::Low => "-",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tier::High => "patrol:trusted",
            Tier::Medium => "patrol:suspicious",
            Tier::Low => "patrol:spam",
        }
    }

    pub fn label_color(self) -> &'static str {
        match self {
            Tier::High => "0e8a16",
            Tier::Medium => "e4a012",
            Tier::Low => "d93f0b",
        }
    }

    pub fn label_description(self) -> &'static str {
        match self {
            Tier::High => "PR author and content look trustworthy",
            Tier::Medium => "PR has some suspicious signals and needs careful review",
            Tier::Low => "PR is likely spam and should be closed",
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
