use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use governor::clock::DefaultClock;
use governor::state::InMemoryState;
use governor::state::direct::NotKeyed;
use governor::{Quota, RateLimiter};

/// A shared, Arc-wrapped token bucket rate limiter.
///
/// Clone the Arc to share the same rate limit budget across multiple tasks.
pub type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Create a rate limiter that allows one operation per `interval`.
///
/// Returns an `Arc`-wrapped governor `RateLimiter` with burst size 1.
/// Panics if `interval` is zero (nonsensical for a rate limiter).
pub fn make_rate_limiter(interval: Duration) -> SharedRateLimiter {
    let quota = Quota::with_period(interval)
        .expect("interval must be non-zero")
        .allow_burst(NonZeroU32::new(1).unwrap());
    Arc::new(RateLimiter::direct(quota))
}

/// Create a rate limiter configured for arXiv's 3-second request interval.
pub fn make_arxiv_limiter() -> SharedRateLimiter {
    make_rate_limiter(Duration::from_secs(3))
}

/// Create a rate limiter configured for InspireHEP's 350ms request interval.
pub fn make_inspirehep_limiter() -> SharedRateLimiter {
    make_rate_limiter(Duration::from_millis(350))
}

/// Wait until the rate limiter grants a token.
///
/// Suspends the current task until a token is available. Suitable for use
/// before each API request to enforce the configured rate limit.
pub async fn wait_for_token(limiter: &SharedRateLimiter) {
    limiter.until_ready().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_rate_limiter_allows_first() {
        // The first token should be immediately available.
        let limiter = make_rate_limiter(Duration::from_millis(500));
        let start = Instant::now();
        wait_for_token(&limiter).await;
        let elapsed = start.elapsed();
        // First token must complete in well under the interval.
        assert!(
            elapsed < Duration::from_millis(200),
            "first token should be immediate, elapsed: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_second() {
        // After consuming the first token the second must wait ~interval.
        let interval = Duration::from_millis(300);
        let limiter = make_rate_limiter(interval);

        // Consume the first token immediately.
        wait_for_token(&limiter).await;

        // Second token should block for approximately `interval`.
        let start = Instant::now();
        wait_for_token(&limiter).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed >= Duration::from_millis(200),
            "second token should block for ~{interval:?}, elapsed: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_arxiv_default() {
        // make_arxiv_limiter produces a 3-second interval limiter.
        // We verify this by checking that the second call blocks > 2s.
        // To keep the test fast we only confirm the limiter is NOT immediately ready
        // after the first token is consumed.
        let limiter = make_arxiv_limiter();
        wait_for_token(&limiter).await; // consume first token

        // The next check should fail (not yet replenished).
        let check_result = limiter.check();
        assert!(
            check_result.is_err(),
            "second check within 3s window should be rate-limited"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_inspirehep_default() {
        // make_inspirehep_limiter produces a 350ms interval limiter.
        let limiter = make_inspirehep_limiter();
        wait_for_token(&limiter).await; // consume first token

        // Immediately after, another check should be rate-limited.
        let check_result = limiter.check();
        assert!(
            check_result.is_err(),
            "second check within 350ms window should be rate-limited"
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_clone_shares_state() {
        // Cloning the Arc should share state — a token consumed on the clone
        // is also consumed for the original.
        let interval = Duration::from_millis(300);
        let limiter = make_rate_limiter(interval);
        let limiter_clone = Arc::clone(&limiter);

        // Consume on clone.
        wait_for_token(&limiter_clone).await;

        // The original should now be rate-limited.
        let start = Instant::now();
        wait_for_token(&limiter).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed >= Duration::from_millis(200),
            "original limiter should block after clone consumed token, elapsed: {elapsed:?}"
        );
    }
}
