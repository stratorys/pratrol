/// Sanitization helpers for untrusted text embedded in GitHub Markdown
/// comments.
///
/// Two levels of sanitization:
/// - `sanitize_plain_text`: for AI-generated fields (summary, `key_signal`,
///   `recommendation`) that are placed inside pre-defined Markdown structures.
///   HTML-escapes via `askama_escape` and neutralizes `@mentions`.
/// - `sanitize_markdown_text`: for external content (PR titles) that may
///   contain attacker-crafted Markdown. Uses `ammonia` to strip all
///   HTML/Markdown structure, then neutralizes `@mentions`.
use askama_escape::{
    Html,
    escape,
};

/// Maximum character length for AI-generated text fields.
const PLAIN_TEXT_LIMIT: usize = 300;

/// Maximum character length for external Markdown fields (PR titles).
const MARKDOWN_TEXT_LIMIT: usize = 200;

/// Sanitize an AI-generated plain-text field for safe embedding in a Markdown
/// comment.
///
/// Steps:
/// 1. Collapse whitespace (neutralizes newline injection).
/// 2. Truncate to `PLAIN_TEXT_LIMIT` characters.
/// 3. HTML-entity-escape via `askama_escape` (`&`, `<`, `>`, `"`, `'`).
/// 4. Neutralize `@mentions` with a zero-width space.
pub fn sanitize_plain_text(input: &str) -> String {
    let collapsed = collapse_whitespace(input);
    let truncated = truncate_chars(&collapsed, PLAIN_TEXT_LIMIT);
    let escaped = escape(&truncated, Html).to_string();

    neutralize_mentions(&escaped)
}

/// Sanitize untrusted Markdown/HTML content (e.g. external PR titles) to plain
/// text.
///
/// Steps:
/// 1. Collapse whitespace.
/// 2. Truncate to `MARKDOWN_TEXT_LIMIT` characters.
/// 3. Strip all HTML via `ammonia` with an empty tag whitelist.
/// 4. Escape Markdown link syntax (`[`, `]`, `(`, `)`) to prevent link
///    injection.
/// 5. Neutralize `@mentions` with a zero-width space.
pub fn sanitize_markdown_text(input: &str) -> String {
    let collapsed = collapse_whitespace(input);
    let truncated = truncate_chars(&collapsed, MARKDOWN_TEXT_LIMIT);

    let cleaned = ammonia::Builder::new()
        .tags(std::collections::HashSet::new())
        .clean(&truncated)
        .to_string();

    let escaped = escape_markdown_links(&cleaned);
    neutralize_mentions(&escaped)
}

/// Escape Markdown link syntax characters to prevent injection of clickable
/// links.
fn escape_markdown_links(input: &str) -> String {
    input
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

/// Replace `@` with `@\u{200B}` to break GitHub mention parsing.
fn neutralize_mentions(input: &str) -> String { input.replace('@', "@\u{200B}") }

/// Collapse all consecutive whitespace into a single space.
fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Unicode-aware truncation to `max_chars` characters, appending `…` if
/// truncated.
fn truncate_chars(
    input: &str,
    max_chars: usize,
) -> String {
    let output = input.chars().take(max_chars).collect::<String>();
    let truncated = input.chars().count() > max_chars;

    if truncated {
        format!("{output}…")
    } else {
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_escapes_html_entities() {
        let result = sanitize_plain_text("a & b < c > d");
        assert!(
            !result.contains("& "),
            "raw ampersand should be escaped: {result}"
        );
        assert!(
            !result.contains('<'),
            "raw less-than should be escaped: {result}"
        );
        assert!(
            !result.contains('>'),
            "raw greater-than should be escaped: {result}"
        );
    }

    #[test]
    fn plain_text_neutralizes_mentions() {
        let result = sanitize_plain_text("ping @security-team now");
        assert!(
            result.contains("@\u{200B}security-team"),
            "mention should be neutralized: {result}"
        );
    }

    #[test]
    fn plain_text_collapses_whitespace() {
        let result = sanitize_plain_text("line1\n\nline2\ttab");
        assert!(
            !result.contains('\n'),
            "newlines should be collapsed: {result}"
        );
        assert!(!result.contains('\t'), "tabs should be collapsed: {result}");
        assert!(
            result.contains("line1 line2 tab"),
            "whitespace should be single spaces: {result}"
        );
    }

    #[test]
    fn plain_text_truncates_long_input() {
        let long = "a".repeat(400);
        let result = sanitize_plain_text(&long);
        assert_eq!(
            result.chars().count(),
            PLAIN_TEXT_LIMIT + 1,
            "should be truncated to limit + ellipsis"
        );
        assert!(result.ends_with('…'), "should end with ellipsis: {result}");
    }

    #[test]
    fn plain_text_truncates_unicode_safely() {
        let long = "é".repeat(301);
        let result = sanitize_plain_text(&long);
        assert!(
            result.contains('…'),
            "unicode input should truncate with ellipsis"
        );
    }

    #[test]
    fn plain_text_escapes_script_tags() {
        let result = sanitize_plain_text("<script>alert(1)</script>");
        assert!(
            !result.contains("<script>"),
            "script tags should be escaped: {result}"
        );
        assert!(
            result.contains("alert(1)"),
            "text content should remain: {result}"
        );
    }

    #[test]
    fn markdown_text_strips_dangerous_html() {
        let result = sanitize_markdown_text("<script>alert(1)</script>");
        assert!(
            !result.contains("<script>"),
            "script tags should be stripped: {result}"
        );
    }

    #[test]
    fn markdown_text_allows_safe_text_content() {
        let result = sanitize_markdown_text("<b>bold</b> text");
        assert!(
            result.contains("bold"),
            "text content should remain: {result}"
        );
    }

    #[test]
    fn markdown_text_neutralizes_mentions() {
        let result = sanitize_markdown_text("ping @security-team");
        assert!(
            result.contains("@\u{200B}security-team"),
            "mention should be neutralized: {result}"
        );
    }

    #[test]
    fn markdown_text_truncates() {
        let long = "x".repeat(300);
        let result = sanitize_markdown_text(&long);
        assert_eq!(
            result.chars().count(),
            MARKDOWN_TEXT_LIMIT + 1,
            "should truncate to markdown limit + ellipsis"
        );
    }

    #[test]
    fn markdown_text_escapes_link_syntax() {
        let result = sanitize_markdown_text("[click me](javascript:evil)");
        assert!(
            result.contains("\\[click me\\]"),
            "brackets should be escaped to prevent Markdown links: {result}"
        );
        assert!(
            result.contains("\\(javascript:evil\\)"),
            "parentheses should be escaped to prevent Markdown links: {result}"
        );
    }
}
