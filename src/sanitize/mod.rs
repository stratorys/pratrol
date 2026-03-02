/// Centralized input sanitization for untrusted text rendered in GitHub
/// comments.
mod text;
mod url;

pub use text::{
    sanitize_markdown_text,
    sanitize_plain_text,
};
pub use url::sanitize_url;
