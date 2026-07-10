use std::time::Duration;

pub(super) const DEDUP_TTL: Duration = Duration::from_mins(10);

pub(super) const DEDUP_CACHE_CAPACITY: u64 = 10_000;
