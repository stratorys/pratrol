# Pratrol

Open-source maintainers drown in pull requests they never asked for. Spam PRs, drive-by contributions from unknown accounts, and low-effort changes waste reviewer time and pollute commit history. The problem scales with popularity — the more visible a project, the more noise lands in the PR queue.

Pratrol is a GitHub App that triages pull requests automatically. It scores every incoming PR across two dimensions — **contributor trust** and **code quality** — then posts a triage brief directly on the PR so maintainers can skip the noise and focus on real contributions.

## What Pratrol catches

**Unknown or suspicious authors.** New GitHub accounts, zero contribution history, no organization memberships. Pratrol builds a contributor profile from public signals and flags accounts that don't look like real contributors.

**Low-quality or incoherent changes.** Diffs that don't match commit messages, risky patterns, or code that looks auto-generated. Pratrol sends the diff to an LLM for structural analysis and surfaces a quality score alongside the contributor profile.

**Repeat offenders.** Authors who have had multiple PRs closed without merge — in the same repo or across GitHub. Pratrol queries PR history and applies a score penalty when it detects a pattern of rejected contributions.

## How it works

When a PR is opened, Pratrol receives the webhook and runs three parallel assessments:

- **Contributor context** — account age, public repos, followers, contribution activity, merge history in the target repo and elsewhere, organization memberships.
- **Code analysis** — diff and commit messages are sent to Mistral for coherence, quality, risk, and suspicious pattern detection.
- **Prior history** — searches GitHub for closed-without-merge PRs by the same author or with similar titles.

The results are combined into a single confidence score (0–100) and tier (High / Medium / Low). Pratrol posts the triage brief as a PR review comment and applies a label (`patrol:trusted`, `patrol:suspicious`, or `patrol:spam`). Repeat offenders get an additional `patrol:repeat-offender` label.

The entire pipeline is stateless — no database, no external storage. Every assessment is computed from live GitHub and Mistral API calls.

## License

MPL-2.0
