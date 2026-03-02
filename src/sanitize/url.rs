/// Sanitize an untrusted URL for safe embedding in a Markdown link.
///
/// Returns `None` if the URL does not use the `https://` scheme, preventing
/// `javascript:`, `data:`, and other dangerous URI schemes.
///
/// Whitespace is stripped to prevent Markdown line-break injection inside
/// `[text](url)`.
pub fn sanitize_url(input: &str) -> Option<String> {
    let url: String = input.split_whitespace().collect();

    if url.starts_with("https://") {
        Some(url)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_url() {
        let result = sanitize_url("https://github.com/o/r/pull/42");
        assert_eq!(result, Some("https://github.com/o/r/pull/42".to_owned()));
    }

    #[test]
    fn strips_whitespace_from_url() {
        let result = sanitize_url("https://github.com/o/r/\n pull/42");
        assert_eq!(result, Some("https://github.com/o/r/pull/42".to_owned()));
    }

    #[test]
    fn rejects_http_url() {
        assert!(
            sanitize_url("http://example.com").is_none(),
            "http should be rejected"
        );
    }

    #[test]
    fn rejects_javascript_url() {
        assert!(
            sanitize_url("javascript:alert(1)").is_none(),
            "javascript scheme should be rejected"
        );
    }

    #[test]
    fn rejects_data_url() {
        assert!(
            sanitize_url("data:text/html,<h1>hi</h1>").is_none(),
            "data scheme should be rejected"
        );
    }

    #[test]
    fn rejects_empty_string() {
        assert!(sanitize_url("").is_none(), "empty URL should be rejected");
    }
}
