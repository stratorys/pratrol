use std::time::Duration;

pub(super) const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) const READ_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) const COMMIT_MESSAGE_MAX_CHARS: usize = 500;

pub(super) const MAX_REVIEW_PAGES: u32 = 3;

pub(super) const PER_PAGE: u8 = 100;
