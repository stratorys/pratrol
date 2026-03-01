use std::fmt;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TriageId(Uuid);

impl TriageId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl fmt::Display for TriageId {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct TriageRequest {
    pub id: TriageId,
    pub installation_id: u64,
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
    pub author_login: String,
}
