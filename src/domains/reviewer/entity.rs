pub struct CodeOwnerRule {
    pub pattern: String,
    pub owners: Vec<String>,
}

pub struct ReviewerCandidate {
    pub login: String,
    pub score: f64,
}
