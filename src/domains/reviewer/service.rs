use std::collections::{
    HashMap,
    HashSet,
};

use tracing::info;

use super::entity::{
    CodeOwnerRule,
    ReviewerCandidate,
};
use super::error::ReviewerError;
use crate::ports::github::GitHubClient;

const MAX_FILES_FOR_HISTORY: usize = 20;
const HISTORY_LIMIT: u32 = 10;
const OWNERSHIP_SCORE: f64 = 10.0;
const HISTORY_BASE_SCORE: f64 = 5.0;
const HISTORY_DECAY: f64 = 0.8;
const MAX_REVIEWERS: usize = 2;

const IGNORED_EXTENSIONS: &[&str] = &[
    "lock", "sum", "png", "jpg", "jpeg", "gif", "svg", "ico", "woff", "woff2", "ttf", "eot", "mp4",
    "webm", "mp3", "ogg", "pdf", "zip", "tar", "gz", "br", "map", "min.js", "min.css",
];

const IGNORED_FILENAMES: &[&str] = &[
    ".gitattributes",
    ".gitignore",
    ".editorconfig",
    ".mailmap",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "Gemfile.lock",
    "poetry.lock",
    "composer.lock",
    "go.sum",
    "flake.lock",
    "shrinkwrap.yaml",
    "bun.lockb",
];

pub struct ReviewerService;

impl ReviewerService {
    pub fn new() -> Self { Self }

    pub async fn assign_reviewers<C: GitHubClient>(
        &self,
        client: &C,
        owner: &str,
        repo: &str,
        pr_number: u64,
        author_login: &str,
    ) -> Result<(), ReviewerError> {
        let codeowners_content = match client.fetch_codeowners(owner, repo).await? {
            Some(content) => content,
            None => return Ok(()),
        };

        let changed_files: Vec<String> = client
            .fetch_pr_files(owner, repo, pr_number)
            .await?
            .into_iter()
            .filter(|f| is_interesting_file(f))
            .collect();
        if changed_files.is_empty() {
            return Ok(());
        }

        let rules = parse_codeowners(&codeowners_content);

        let mut all_owners: HashSet<String> = HashSet::new();
        let mut file_owners_map: HashMap<&str, Vec<String>> = HashMap::new();

        for file in &changed_files {
            let owners = find_owners(&rules, file);
            for file_owner in &owners {
                all_owners.insert(file_owner.clone());
            }
            file_owners_map.insert(file, owners);
        }

        let (individual_owners, teams) = partition_owners(&all_owners);
        let individual_set: HashSet<&str> = individual_owners.iter().map(String::as_str).collect();

        let files_to_fetch: Vec<&str> = changed_files
            .iter()
            .take(MAX_FILES_FOR_HISTORY)
            .map(String::as_str)
            .collect();

        let history_futures: Vec<_> = files_to_fetch
            .iter()
            .map(|file| client.fetch_file_contributors(owner, repo, file, HISTORY_LIMIT))
            .collect();

        let histories = futures::future::join_all(history_futures).await;

        let mut scores: HashMap<String, f64> = HashMap::new();

        for (i, file) in changed_files.iter().enumerate() {
            if let Some(owners) = file_owners_map.get(file.as_str()) {
                for file_owner in owners {
                    if individual_set.contains(file_owner.as_str()) {
                        *scores.entry(file_owner.clone()).or_default() += OWNERSHIP_SCORE;
                    }
                }
            }

            if i < files_to_fetch.len()
                && let Ok(contributors) = &histories[i]
            {
                for (idx, contributor) in contributors.iter().enumerate() {
                    *scores.entry(contributor.clone()).or_default() +=
                        HISTORY_BASE_SCORE * HISTORY_DECAY.powi(idx as i32);
                }
            }
        }

        let mut candidates: Vec<ReviewerCandidate> = scores
            .into_iter()
            .map(|(login, score)| ReviewerCandidate {
                login,
                score,
            })
            .collect();

        filter_candidates(&mut candidates, author_login);

        candidates.sort_by(|a, b| b.score.total_cmp(&a.score));

        let top_users: Vec<String> = candidates
            .iter()
            .take(MAX_REVIEWERS)
            .map(|c| c.login.clone())
            .collect();

        let team_reviewers: Vec<String> = if top_users.len() < MAX_REVIEWERS {
            let remaining = MAX_REVIEWERS - top_users.len();
            teams.into_iter().take(remaining).collect()
        } else {
            Vec::new()
        };

        if top_users.is_empty() && team_reviewers.is_empty() {
            info!(message = "No reviewer candidates found, skipping assignment.");
            return Ok(());
        }

        client
            .request_reviewers(owner, repo, pr_number, &top_users, &team_reviewers)
            .await?;

        info!(
            message = "Assigned reviewers.",
            users = ?top_users,
            teams = ?team_reviewers,
        );

        Ok(())
    }
}

pub fn parse_codeowners(content: &str) -> Vec<CodeOwnerRule> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let mut parts = trimmed.split_whitespace();
            let pattern = parts.next()?;
            let owners: Vec<String> = parts
                .filter(|part| part.starts_with('@'))
                .map(|part| part.trim_start_matches('@').to_owned())
                .collect();

            if owners.is_empty() {
                return None;
            }

            Some(CodeOwnerRule {
                pattern: pattern.to_owned(),
                owners,
            })
        })
        .collect()
}

pub fn matches_codeowner_pattern(
    pattern: &str,
    file_path: &str,
) -> bool {
    let glob_pattern = codeowner_pattern_to_glob(pattern);
    glob_match::glob_match(&glob_pattern, file_path)
}

fn codeowner_pattern_to_glob(pattern: &str) -> String {
    if !pattern.contains('/') {
        format!("**/{pattern}")
    } else if pattern.ends_with('/') {
        let trimmed = pattern.trim_start_matches('/');
        let dir = trimmed.trim_end_matches('/');
        if pattern.starts_with('/') {
            format!("{dir}/**")
        } else {
            format!("**/{dir}/**")
        }
    } else if let Some(anchored) = pattern.strip_prefix('/') {
        anchored.to_owned()
    } else {
        pattern.to_owned()
    }
}

pub fn is_interesting_file(path: &str) -> bool {
    let filename = path.rsplit('/').next().unwrap_or(path);

    if IGNORED_FILENAMES.contains(&filename) {
        return false;
    }

    for ext in IGNORED_EXTENSIONS {
        if filename.ends_with(&format!(".{ext}")) {
            return false;
        }
    }

    true
}

fn find_owners(
    rules: &[CodeOwnerRule],
    file_path: &str,
) -> Vec<String> {
    for rule in rules.iter().rev() {
        if matches_codeowner_pattern(&rule.pattern, file_path) {
            return rule.owners.clone();
        }
    }
    Vec::new()
}

pub fn partition_owners(owners: &HashSet<String>) -> (Vec<String>, Vec<String>) {
    let mut users = Vec::new();
    let mut teams = Vec::new();

    for owner in owners {
        if let Some((_org, team)) = owner.split_once('/') {
            teams.push(team.to_owned());
        } else {
            users.push(owner.clone());
        }
    }

    (users, teams)
}

pub fn filter_candidates(
    candidates: &mut Vec<ReviewerCandidate>,
    author: &str,
) {
    candidates.retain(|c| c.login != author && !c.login.ends_with("[bot]"));
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use async_trait::async_trait;

    use super::*;
    use crate::ports::github::{
        GitHubClient,
        GitHubError,
        UserInfo,
    };

    struct MockReviewerClient {
        codeowners: Option<String>,
        files: Vec<String>,
        contributors: HashMap<String, Vec<String>>,
        requested_users: std::sync::Mutex<Vec<String>>,
        requested_teams: std::sync::Mutex<Vec<String>>,
    }

    impl MockReviewerClient {
        fn new(
            codeowners: Option<&str>,
            files: Vec<&str>,
            contributors: HashMap<&str, Vec<&str>>,
        ) -> Self {
            Self {
                codeowners: codeowners.map(|s| s.to_owned()),
                files: files.into_iter().map(|s| s.to_owned()).collect(),
                contributors: contributors
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v.into_iter().map(|s| s.to_owned()).collect()))
                    .collect(),
                requested_users: std::sync::Mutex::new(Vec::new()),
                requested_teams: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl GitHubClient for MockReviewerClient {
        async fn fetch_user(
            &self,
            _login: &str,
        ) -> Result<UserInfo, GitHubError> {
            Ok(UserInfo {
                account_age_days: 365,
                public_repos: 10,
                followers: 5,
            })
        }

        async fn fetch_events_count(
            &self,
            _login: &str,
        ) -> Result<u32, GitHubError> {
            Ok(0)
        }

        async fn fetch_orgs_count(
            &self,
            _login: &str,
        ) -> Result<u32, GitHubError> {
            Ok(0)
        }

        async fn fetch_merged_prs(
            &self,
            _login: &str,
            _owner: &str,
            _repo: &str,
        ) -> Result<u32, GitHubError> {
            Ok(0)
        }

        async fn fetch_merged_prs_global(
            &self,
            _login: &str,
        ) -> Result<u32, GitHubError> {
            Ok(0)
        }

        async fn fetch_diff(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
        ) -> Result<String, GitHubError> {
            Ok(String::new())
        }

        async fn fetch_commits(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
        ) -> Result<Vec<String>, GitHubError> {
            Ok(Vec::new())
        }

        async fn post_review(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
            _body: &str,
        ) -> Result<(), GitHubError> {
            Ok(())
        }

        async fn fetch_codeowners(
            &self,
            _owner: &str,
            _repo: &str,
        ) -> Result<Option<String>, GitHubError> {
            Ok(self.codeowners.clone())
        }

        async fn fetch_pr_files(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
        ) -> Result<Vec<String>, GitHubError> {
            Ok(self.files.clone())
        }

        async fn fetch_file_contributors(
            &self,
            _owner: &str,
            _repo: &str,
            path: &str,
            _limit: u32,
        ) -> Result<Vec<String>, GitHubError> {
            Ok(self.contributors.get(path).cloned().unwrap_or_default())
        }

        async fn request_reviewers(
            &self,
            _owner: &str,
            _repo: &str,
            _pr_number: u64,
            users: &[String],
            teams: &[String],
        ) -> Result<(), GitHubError> {
            if let Ok(mut locked) = self.requested_users.lock() {
                *locked = users.to_vec();
            }
            if let Ok(mut locked) = self.requested_teams.lock() {
                *locked = teams.to_vec();
            }
            Ok(())
        }
    }

    #[test]
    fn test_parse_codeowners_empty() {
        let rules = parse_codeowners("");
        assert!(rules.is_empty(), "empty input yields no rules");
    }

    #[test]
    fn test_parse_codeowners_comments_and_blanks() {
        let content = "# comment\n\n  # another comment\n";
        let rules = parse_codeowners(content);
        assert!(rules.is_empty(), "comments and blanks yield no rules");
    }

    #[test]
    fn test_parse_codeowners_rules() {
        let content = "*.rs @alice @bob\n/docs/ @charlie\n";
        let rules = parse_codeowners(content);
        assert_eq!(rules.len(), 2, "should parse two rules");
        assert_eq!(rules[0].pattern, "*.rs", "first pattern");
        assert_eq!(rules[0].owners, vec!["alice", "bob"], "first owners");
        assert_eq!(rules[1].pattern, "/docs/", "second pattern");
        assert_eq!(rules[1].owners, vec!["charlie"], "second owners");
    }

    #[test]
    fn test_parse_codeowners_teams() {
        let content = "*.js @org/frontend-team @alice\n";
        let rules = parse_codeowners(content);
        assert_eq!(rules.len(), 1, "should parse one rule");
        assert_eq!(
            rules[0].owners,
            vec!["org/frontend-team", "alice"],
            "should include team and user"
        );
    }

    #[test]
    fn test_parse_codeowners_no_owners_skipped() {
        let content = "*.rs\n*.js @alice\n";
        let rules = parse_codeowners(content);
        assert_eq!(rules.len(), 1, "line without owners should be skipped");
        assert_eq!(rules[0].pattern, "*.js", "only valid rule kept");
    }

    #[test]
    fn test_matches_glob_star() {
        assert!(
            matches_codeowner_pattern("*.rs", "src/main.rs"),
            "*.rs should match src/main.rs"
        );
        assert!(
            matches_codeowner_pattern("*.rs", "main.rs"),
            "*.rs should match main.rs at root"
        );
        assert!(
            !matches_codeowner_pattern("*.rs", "src/main.js"),
            "*.rs should not match .js files"
        );
    }

    #[test]
    fn test_matches_directory_anchored() {
        assert!(
            matches_codeowner_pattern("/docs/", "docs/readme.md"),
            "/docs/ should match docs/readme.md"
        );
        assert!(
            matches_codeowner_pattern("/docs/", "docs/sub/file.txt"),
            "/docs/ should match nested files"
        );
        assert!(
            !matches_codeowner_pattern("/docs/", "src/docs/readme.md"),
            "/docs/ should not match non-root docs"
        );
    }

    #[test]
    fn test_matches_directory_unanchored() {
        assert!(
            matches_codeowner_pattern("docs/", "docs/readme.md"),
            "docs/ should match root docs"
        );
        assert!(
            matches_codeowner_pattern("docs/", "src/docs/readme.md"),
            "docs/ should match nested docs"
        );
    }

    #[test]
    fn test_matches_anchored_path() {
        assert!(
            matches_codeowner_pattern("/src/main.rs", "src/main.rs"),
            "/src/main.rs should match exactly"
        );
        assert!(
            !matches_codeowner_pattern("/src/main.rs", "other/src/main.rs"),
            "/src/main.rs should not match non-root path"
        );
    }

    #[test]
    fn test_matches_relative_path_with_slash() {
        assert!(
            matches_codeowner_pattern("src/*.rs", "src/main.rs"),
            "src/*.rs should match src/main.rs"
        );
        assert!(
            !matches_codeowner_pattern("src/*.rs", "src/sub/main.rs"),
            "src/*.rs should not match nested files"
        );
    }

    #[test]
    fn test_find_owners_last_match_wins() {
        let rules = parse_codeowners("*.rs @alice\n/src/main.rs @bob\n");
        let owners = find_owners(&rules, "src/main.rs");
        assert_eq!(owners, vec!["bob"], "last matching rule should win");
    }

    #[test]
    fn test_find_owners_no_match() {
        let rules = parse_codeowners("*.py @alice\n");
        let owners = find_owners(&rules, "src/main.rs");
        assert!(owners.is_empty(), "no matching rule returns empty");
    }

    #[test]
    fn test_partition_owners_users_and_teams() {
        let owners: HashSet<String> = ["alice", "org/backend", "bob", "corp/infra"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let (mut users, mut teams) = partition_owners(&owners);
        users.sort();
        teams.sort();
        assert_eq!(users, vec!["alice", "bob"], "individual users");
        assert_eq!(teams, vec!["backend", "infra"], "team slugs extracted");
    }

    #[test]
    fn test_is_interesting_file_accepts_source_files() {
        assert!(is_interesting_file("src/main.rs"), "source file");
        assert!(is_interesting_file("lib/utils.js"), "js file");
        assert!(is_interesting_file("README.md"), "markdown");
        assert!(is_interesting_file("Cargo.toml"), "config toml");
    }

    #[test]
    fn test_is_interesting_file_rejects_lock_files() {
        assert!(!is_interesting_file("Cargo.lock"), "Cargo.lock");
        assert!(!is_interesting_file("package-lock.json"), "package-lock");
        assert!(!is_interesting_file("yarn.lock"), "yarn.lock");
        assert!(!is_interesting_file("pnpm-lock.yaml"), "pnpm-lock");
        assert!(!is_interesting_file("Gemfile.lock"), "Gemfile.lock");
        assert!(!is_interesting_file("go.sum"), "go.sum");
        assert!(!is_interesting_file("flake.lock"), "flake.lock");
        assert!(!is_interesting_file("bun.lockb"), "bun.lockb");
    }

    #[test]
    fn test_is_interesting_file_rejects_dotfiles() {
        assert!(!is_interesting_file(".gitattributes"), ".gitattributes");
        assert!(!is_interesting_file(".gitignore"), ".gitignore");
        assert!(!is_interesting_file(".editorconfig"), ".editorconfig");
    }

    #[test]
    fn test_is_interesting_file_rejects_binaries_and_assets() {
        assert!(!is_interesting_file("logo.png"), "png");
        assert!(!is_interesting_file("assets/icon.svg"), "svg");
        assert!(!is_interesting_file("fonts/roboto.woff2"), "woff2");
        assert!(!is_interesting_file("bundle.min.js"), "minified js");
        assert!(!is_interesting_file("style.min.css"), "minified css");
        assert!(!is_interesting_file("app.js.map"), "source map");
    }

    #[test]
    fn test_is_interesting_file_nested_lock_files() {
        assert!(
            !is_interesting_file("subdir/package-lock.json"),
            "nested lock"
        );
        assert!(
            !is_interesting_file("deep/path/yarn.lock"),
            "deeply nested lock"
        );
    }

    #[test]
    fn test_filter_candidates_removes_author() {
        let mut candidates = vec![
            ReviewerCandidate {
                login: "alice".to_owned(),
                score: 10.0,
            },
            ReviewerCandidate {
                login: "author".to_owned(),
                score: 20.0,
            },
        ];
        filter_candidates(&mut candidates, "author");
        assert_eq!(candidates.len(), 1, "author should be removed");
        assert_eq!(candidates[0].login, "alice", "non-author remains");
    }

    #[test]
    fn test_filter_candidates_removes_bots() {
        let mut candidates = vec![
            ReviewerCandidate {
                login: "alice".to_owned(),
                score: 10.0,
            },
            ReviewerCandidate {
                login: "dependabot[bot]".to_owned(),
                score: 15.0,
            },
        ];
        filter_candidates(&mut candidates, "nobody");
        assert_eq!(candidates.len(), 1, "bot should be removed");
        assert_eq!(candidates[0].login, "alice", "human remains");
    }

    #[test]
    fn test_scoring_ownership_and_history() {
        let mut scores: HashMap<String, f64> = HashMap::new();

        *scores.entry("alice".to_owned()).or_default() += OWNERSHIP_SCORE;

        let history = vec!["bob", "alice"];
        for (idx, contributor) in history.iter().enumerate() {
            *scores.entry((*contributor).to_owned()).or_default() +=
                HISTORY_BASE_SCORE * HISTORY_DECAY.powi(idx as i32);
        }

        let alice_score = scores["alice"];
        let bob_score = scores["bob"];

        let expected_alice = OWNERSHIP_SCORE + HISTORY_BASE_SCORE * HISTORY_DECAY;
        let expected_bob = HISTORY_BASE_SCORE;

        assert!(
            (alice_score - expected_alice).abs() < f64::EPSILON,
            "alice: ownership + second history commit"
        );
        assert!(
            (bob_score - expected_bob).abs() < f64::EPSILON,
            "bob: first history commit only"
        );
        assert!(
            alice_score > bob_score,
            "alice should rank higher due to ownership"
        );
    }

    #[tokio::test]
    async fn test_assign_reviewers_integration() {
        let mut contributors = HashMap::new();
        contributors.insert("src/main.rs", vec!["charlie", "alice"]);

        let client = MockReviewerClient::new(
            Some("*.rs @alice @bob\n"),
            vec!["src/main.rs"],
            contributors,
        );

        let service = ReviewerService::new();
        let result = service
            .assign_reviewers(&client, "owner", "repo", 1, "charlie")
            .await;

        assert!(result.is_ok(), "assign_reviewers should succeed");

        let users = client.requested_users.lock().expect("lock poisoned");
        assert_eq!(users.len(), 2, "should assign 2 reviewers");
        assert_eq!(
            users[0], "alice",
            "alice should be first (ownership + history)"
        );
        assert_eq!(users[1], "bob", "bob should be second (ownership only)");
    }

    #[tokio::test]
    async fn test_assign_reviewers_no_codeowners() {
        let client = MockReviewerClient::new(None, vec!["src/main.rs"], HashMap::new());

        let service = ReviewerService::new();
        let result = service
            .assign_reviewers(&client, "owner", "repo", 1, "author")
            .await;

        assert!(result.is_ok(), "should skip silently without CODEOWNERS");
        let users = client.requested_users.lock().expect("lock poisoned");
        assert!(users.is_empty(), "no reviewers should be requested");
    }

    #[tokio::test]
    async fn test_assign_reviewers_author_only_owner() {
        let client =
            MockReviewerClient::new(Some("*.rs @author\n"), vec!["src/main.rs"], HashMap::new());

        let service = ReviewerService::new();
        let result = service
            .assign_reviewers(&client, "owner", "repo", 1, "author")
            .await;

        assert!(result.is_ok(), "should skip when author is only candidate");
        let users = client.requested_users.lock().expect("lock poisoned");
        assert!(users.is_empty(), "no reviewers should be requested");
    }

    #[tokio::test]
    async fn test_assign_reviewers_fills_with_teams() {
        let client = MockReviewerClient::new(
            Some("*.rs @org/backend-team\n"),
            vec!["src/main.rs"],
            HashMap::new(),
        );

        let service = ReviewerService::new();
        let result = service
            .assign_reviewers(&client, "owner", "repo", 1, "author")
            .await;

        assert!(result.is_ok(), "should succeed with team assignment");
        let teams = client.requested_teams.lock().expect("lock poisoned");
        assert_eq!(teams.len(), 1, "should assign 1 team");
        assert_eq!(teams[0], "backend-team", "team slug should be extracted");
    }
}
