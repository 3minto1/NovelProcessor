use crate::domain::ModelProfile;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub(crate) const RATE_LIMIT_RETRY_EXHAUSTED: &str = "服务商限流重试已耗尽";
pub(crate) const MAX_RATE_LIMIT_RETRIES: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RateLimitScope(String);

impl RateLimitScope {
    pub(crate) fn for_profile(profile: &ModelProfile) -> Self {
        Self(format!(
            "{}|{}|{}|{}",
            profile.id.trim(),
            profile.provider.trim().to_ascii_lowercase(),
            profile
                .base_url
                .trim()
                .trim_end_matches('/')
                .to_ascii_lowercase(),
            profile.model.trim().to_ascii_lowercase()
        ))
    }
}

#[derive(Debug, Clone)]
struct RateLimitState {
    cooldown_until: Instant,
    consecutive_limits: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RateLimitCoordinator {
    states: Arc<Mutex<HashMap<RateLimitScope, RateLimitState>>>,
}

impl RateLimitCoordinator {
    pub(crate) fn cooldown_delay(
        &self,
        scope: &RateLimitScope,
    ) -> Result<Option<Duration>, String> {
        let mut states = self.states.lock().map_err(|error| error.to_string())?;
        let Some(state) = states.get(scope) else {
            return Ok(None);
        };
        let now = Instant::now();
        if state.cooldown_until <= now {
            states.remove(scope);
            Ok(None)
        } else {
            Ok(Some(state.cooldown_until.duration_since(now)))
        }
    }

    pub(crate) fn record_rate_limit(
        &self,
        scope: &RateLimitScope,
        retry_after: Option<Duration>,
        attempt: usize,
    ) -> Result<Duration, String> {
        let delay = retry_after.unwrap_or_else(|| default_backoff_delay(attempt));
        let mut states = self.states.lock().map_err(|error| error.to_string())?;
        let state = states
            .entry(scope.clone())
            .or_insert_with(|| RateLimitState {
                cooldown_until: Instant::now(),
                consecutive_limits: 0,
            });
        state.consecutive_limits = state.consecutive_limits.saturating_add(1);
        state.cooldown_until = Instant::now() + delay;
        Ok(delay)
    }

    pub(crate) fn record_success(&self, scope: &RateLimitScope) -> Result<(), String> {
        let mut states = self.states.lock().map_err(|error| error.to_string())?;
        states.remove(scope);
        Ok(())
    }
}

pub(crate) fn is_rate_limit_retry_exhausted(error: &str) -> bool {
    error.contains(RATE_LIMIT_RETRY_EXHAUSTED)
}

pub(crate) fn parse_retry_after(value: Option<&str>) -> Option<Duration> {
    let seconds = value?.trim().parse::<u64>().ok()?;
    Some(Duration::from_secs(seconds.clamp(1, 600)))
}

pub(crate) fn default_backoff_delay(attempt: usize) -> Duration {
    let base = match attempt {
        0 | 1 => 45,
        2 => 90,
        3 => 180,
        _ => 300,
    };
    Duration::from_secs(base)
}
