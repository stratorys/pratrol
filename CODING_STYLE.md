# Pratrol Rust coding style

This document defines the conventions used in the Pratrol codebase. The goal is
to keep code explicit, cohesive, predictable under failure, and easy to navigate
without introducing framework-shaped architecture.

## Precedence and scope

Repository-local patterns win over this guide. When you edit an existing file,
match its established patterns. This guide in turn wins over personal preference
and over patterns copied from unrelated code. It applies to production code;
test and generated code follow it where practical, and the few explicit
exceptions are called out per rule, such as `unwrap` in tests.

## Modules and files

Pratrol keeps declarations in `mod.rs`, and the `self_named_module_files` lint in
`Cargo.toml` enforces that. Each `mod.rs` is limited to module declarations and
narrow re-exports, while the implementation lives beside it in single-word named
files such as `workflow.rs`, `router.rs`, `entity.rs`, `traits.rs`,
`connector.rs`, or `error.rs`. Module-specific errors belong in `error.rs`. Keep
every module focused on a single business concept or technical responsibility,
and avoid bucket modules such as `utils`, `helpers`, `common`, `misc`, `actions`,
or `services`.

```text
src/
  app/
    triage/
      mod.rs          declarations only
      workflow.rs
      load.rs
      assess.rs
      decide.rs
  domains/
    scoring/
      mod.rs
  connectors/
    github/
      mod.rs
      connector.rs
      client.rs
```

Here `triage/mod.rs` declares those modules, and the behavior lives in the named
files beside it.

## Project organization

Code is organized by business capability. An `app/<capability>` module
coordinates workflows and external effects, a `domains/<capability>` module holds
deterministic business rules and domain types, and a `connectors/<provider>`
module implements external APIs and infrastructure. `main.rs` is the composition
root that constructs dependencies and starts the application.

In Pratrol, `app::triage` coordinates loading, assessment, decision, and
publication, while `app::review` owns review publication and `app::labels` owns
label behavior. The deterministic rules live in the domains: `domains::scoring`
owns score calculations, `domains::history` owns history rules,
`domains::analysis` owns prompt and response semantics, and `domains::comment`
owns comment rendering. External systems are reached through
`connectors::github`, `connectors::mistral`, and `connectors::gemma`.

Provider-specific types and errors stay inside connectors, and no workflow should
be collapsed into a single `Service` type or file. `Triage::execute` instead
reads as a sequence of domain steps:

```rust
let Some(context) = load::context(self.github.as_ref(), request).await? else {
    return Ok(());
};

let assessment = assess::pull_request(&self.harness, &context).await;
let decision = decide::triage(&context, assessment);
let publication = decide::review(&context, &decision)?;

crate::app::review::publish::review(context.client.as_ref(), publication).await
```

## Imports and paths

Do not use wildcard imports or leading `::` in paths. Group imports as standard
library, then third-party crates, then `crate::`, with a blank line between the
groups, as required by `group_imports = StdExternalCrate` in `rustfmt.toml`.
Rename a local module when it would otherwise be ambiguous with an external
crate.

```rust
use std::time::Duration;

use tracing::error;

use crate::config::Config;
use crate::domains::llm::error::LlmError;
```

## Functions and control flow

Prefer expression-oriented code and early returns, and reach for intermediate
variables when they clarify data flow. Extract deeply nested control flow into
focused functions, and use `match` for meaningful domain decisions and enum
interpretation. Prefer iterators for simple transformations, but use a mutable
accumulator when it is materially clearer or more efficient. Do not silence
`clippy::too_many_arguments`; introduce a parameter struct instead.

```rust
let result = match input {
    Some(value) => parse(value),
    None => return Err(Error::MissingInput),
};
```

## Ownership and mutation

Prefer immutable bindings and shadowing, and mutate only when an API requires it
or when in-place construction is clearer or more efficient. Avoid interior
mutability and shared mutable state without a clear ownership or concurrency
requirement, prefer message passing over shared mutable state, and prefer `&T`
over `Arc<T>` when shared ownership is unnecessary. Treat every clone as an
intentional ownership decision. Use owned `String` and `Vec<T>` for structs and
long-lived data, but take `&str` and `&[T]` as parameters when ownership is
unnecessary and no explicit lifetime is required; avoid explicit lifetimes unless
borrowing is a real design requirement.

```rust
let value = load()?;
let value = normalize(value);
```

## Types and naming

Prefer named structs and enums over tuples, boolean flags, and loosely related
primitive arguments, and use newtypes for values with distinct meaning or
invariants. Keep fields private when construction must enforce an invariant, and
do not expose provider DTOs outside their connector; return owned domain types
such as `UserInfo` or `CommitInfo` instead. Prefer fixed-width integers such as
`u32` and `u64`, and use `usize` only for true indexes, lengths, memory sizes, or
APIs that require it. Put units last in names, as in `latency_ms_max`,
`retry_delay_s`, and `payload_size_bytes`.

```rust
struct PullRequestNumber(u64);
struct InstallationId(u64);
```

## Errors and panics

Use concrete error enums built with `thiserror`, and keep module errors in
`error.rs`. Do not use `Box<dyn Error>` or `anyhow` in production code. Prefer
`#[from]` for common, unambiguous conversions, and a focused `map_err` when the
mapping is rare or contextual. Always name the error binding `error`, never `e`
or `err`.

Preserve operational failures as errors; do not convert failed I/O, parsing,
validation, or external calls into empty values. A default is acceptable only
when absence is a valid domain state, such as an inapplicable title-history query
or an optional webhook field whose absence has a defined protocol meaning.
Configuration that is required must fail at startup when it is missing or
invalid, and optional configuration is represented with `Option<T>`, as with
`gemma_api_key`. Provider-specific errors stay inside connectors and are mapped
to stable domain errors such as `GitHubError` and `LlmError` before they cross
the boundary.

```rust
let history = load_history(client).await?;
```

The following swallows a real failure and is not acceptable:

```rust
let history = load_history(client)
    .await
    .unwrap_or_else(|_| HistorySignals::empty());
```

Do not use `unwrap()` in production; `unwrap_used` is denied crate-wide.
`expect()` is reserved for startup, where a required resource is missing or a
required invariant cannot hold and the application must not run without it, such
as installing the default crypto provider or the SIGTERM handler; the message
states what was required. Do not use `expect()` to paper over a failure that the
running application could handle and continue past. Do not use `panic!()` except
for a documented, unrecoverable invariant carrying a comment explaining why
recovery is impossible.

## HTTP API errors

The transport boundary exposes a stable `ApiError` that implements
`IntoResponse`. Application errors are converted into `ApiError` where the request
is handled, logged once at that boundary, and never allowed to leak provider
details or sensitive internal information into the response body.

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!(message = "Request failed.", error = %self);

        let status = match &self {
            ApiError::InvalidSignature => StatusCode::UNAUTHORIZED,
            ApiError::InvalidPayload => StatusCode::BAD_REQUEST,
            ApiError::Triage(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}
```

## Tracing

Use structured `tracing` only. Put the human-readable text in a `message` field
that starts with a capital letter and ends with a period, log errors with
`%error` rather than `?error`, and log at the boundary that finally handles the
error rather than repeating the same error across layers.

The levels carry consistent meaning: `error!` marks an operation that failed and
cannot continue, `warn!` marks one that recovered but is degraded, `info!` marks
lifecycle events and meaningful state transitions, and `debug!` and `trace!`
carry diagnostic detail.

```rust
error!(message = "Failed to build HTTP client.", %error);
```

## Traits and dependency injection

Prefer concrete types by default, and introduce a trait only when multiple
implementations are required, a dependency must be replaced in tests, the
implementation is selected at startup, or the interface represents an external
capability. Define traits around capabilities rather than implementation
details, and keep them in a `traits.rs` beside the concept they represent rather
than in a generic `ports.rs` or `interfaces.rs`. Pratrol's `GitHubApp`,
`GitHubClient`, and `Llm` traits all live in their domain `traits.rs`.

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Llm: Send + Sync {
    async fn chat_completion(&self, request: &ChatRequest) -> Result<String, LlmError>;
}
```

Dependencies are constructed explicitly in `main.rs`, without a service locator
or runtime dependency-injection container. Use `Arc<dyn Trait>` only when shared
ownership and dynamic dispatch are both required, as with the `Arc<dyn GitHubApp>`
handed to the triage workflow and the `Arc<dyn Llm>` handed to the harness.

## External effects and async code

External effects belong at the edges, and domain modules stay deterministic and
provider-independent. GitHub API calls live in `connectors::github`, and LLM
protocol calls live in the provider connectors. Do not perform blocking work
directly in async tasks. Prefer message passing between tasks, and use bounded
channels and explicit concurrency limits. Do not give the LLM or a future
autonomous agent unrestricted connector access; agent behavior proposes typed
operations that application code validates and authorizes.

## Resource bounds

Every loop, retry, queue, buffer, batch, pagination, timeout, concurrency limit,
and allocation must be bounded explicitly, and unbounded channels or loops are
used only when the risk is documented. The request body limit and per-request
timeout are kept close to their edge:

```rust
.layer(DefaultBodyLimit::max(256 * 1024))
```

```rust
const REQUEST_TIMEOUT: Duration = Duration::from_mins(1);
```

## Dependencies

Add dependencies with `cargo add` rather than hand-editing `Cargo.toml`, and
review default features so only what the project needs is enabled. The `mistral`
and `gemma` connector features are mutually exclusive.

## Comments and documentation

Do not add new code comments except to document an intentional panic, and keep
existing comments unless the task requires changing them. Prefer clear names,
focused functions, and explicit types over explanatory comments, and update this
document when responsibilities or boundaries change.

## Testing

Focused unit tests live beside the module they exercise, pure domain functions
are tested directly, and external capabilities are mocked through the `mockall`
trait mocks rather than through internal helper functions. Test failure paths
explicitly, including that operational errors remain errors rather than becoming
empty data, and start bug fixes with a failing regression test when practical.
Use integration-test targets for HTTP, routing, and webhook boundaries, and
prefer small, deterministic fixtures. Because `unwrap_used` is denied crate-wide
and there is no `clippy.toml` to exempt tests, tests use `expect()` with a
message; `expect_used` is not denied, so `expect()` is permitted everywhere
today. Tests describe observable behavior rather than implementation details.

## Formatting and checks

Because `mistral` and `gemma` are mutually exclusive, run the checks per
configuration rather than with `--all-features`, and use nightly Rustfmt since
`rustfmt.toml` sets `unstable_features = true`.

```text
cargo +nightly fmt --all -- --check
cargo test --features mistral
cargo test --no-default-features --features gemma
cargo clippy --all-targets --features mistral -- -D warnings
cargo clippy --all-targets --no-default-features --features gemma -- -D warnings
git diff --check
```

## Enforcement

Lints live in `Cargo.toml` under `[lints.rust]` and `[lints.clippy]`, and import
grouping lives in `rustfmt.toml` under nightly. Refer to those files for the exact set;
`[lints.rust]` denies all warnings and `unsafe_code`, and clippy runs with the
`pedantic` group plus stricter lints denied on top.

Some rules in this guide are not mechanically enforced and rely on review.
Because `expect_used` is not denied, `expect()` is not caught by clippy, and the
bans on `panic!`, `anyhow`, and `Box<dyn Error>` are review-enforced rather than
linted.

## Guiding principles

Prefer designs that make ownership and data flow explicit, keep business rules
independent from infrastructure, use small and cohesive modules, preserve errors
instead of hiding them, bound resource use and concurrency, avoid abstractions
until they solve a demonstrated problem, and follow the repository's exact tooling
and validation workflow.
