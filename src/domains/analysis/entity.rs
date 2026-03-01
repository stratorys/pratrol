pub struct AnalysisResult {
    pub code_coherence: f64,
    pub commit_quality: f64,
    pub risk_level: f64,
    pub suspicious_patterns: f64,
    pub summary: String,
    pub key_signal: String,
    pub recommendation: String,
}
