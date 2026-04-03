use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

pub type Limiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

pub struct RateLimiters {
    pub musicbrainz: Limiter,
    pub discogs: Limiter,
    pub lastfm: Limiter,
    pub acoustid: Limiter,
    pub wikipedia: Limiter,
}

impl RateLimiters {
    pub fn new() -> Self {
        Self {
            musicbrainz: make_limiter(1),
            discogs: make_limiter(1),   // 60/min = 1/s efectivo (2 req/track)
            lastfm: make_limiter(5),
            acoustid: make_limiter(3),
            wikipedia: make_limiter(1),
        }
    }
}

fn make_limiter(rps: u32) -> Limiter {
    Arc::new(RateLimiter::direct(
        Quota::per_second(NonZeroU32::new(rps).unwrap()),
    ))
}
