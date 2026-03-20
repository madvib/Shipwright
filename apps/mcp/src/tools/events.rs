use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use std::path::Path;

use crate::requests::ListEventsRequest;

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

/// Parse a relative time string ("1h", "24h", "7d") or ISO 8601 timestamp
/// into a UTC cutoff. Events before this cutoff are excluded.
fn parse_since(raw: &str) -> anyhow::Result<DateTime<Utc>> {
    let s = raw.trim();
    if let Some(h) = s.strip_suffix('h') {
        let hours: i64 = h.parse().map_err(|_| anyhow!("Invalid hours value: '{}'", s))?;
        return Ok(Utc::now() - Duration::hours(hours));
    }
    if let Some(d) = s.strip_suffix('d') {
        let days: i64 = d.parse().map_err(|_| anyhow!("Invalid days value: '{}'", s))?;
        return Ok(Utc::now() - Duration::days(days));
    }
    s.parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| anyhow!("Could not parse '{}' as a timestamp: {}", s, e))
        })
}

pub fn list_events(project_dir: &Path, req: ListEventsRequest) -> String {
    let ship_dir = project_dir.join(".ship");

    let since_cutoff = match req.since.as_deref().map(parse_since) {
        Some(Err(e)) => return format!("Error parsing since: {}", e),
        Some(Ok(dt)) => Some(dt),
        None => None,
    };

    let limit = req.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let mut events = match runtime::events::read_events(&ship_dir) {
        Ok(ev) => ev,
        Err(e) => return format!("Error reading events: {}", e),
    };

    if let Some(cutoff) = since_cutoff {
        events.retain(|e| e.timestamp >= cutoff);
    }
    if let Some(ref actor_filter) = req.actor {
        events.retain(|e| e.actor.contains(actor_filter.as_str()));
    }
    if let Some(ref entity_filter) = req.entity {
        let ef = entity_filter.to_ascii_lowercase();
        events.retain(|e| format!("{:?}", e.entity).to_ascii_lowercase().contains(&ef));
    }
    if let Some(ref action_filter) = req.action {
        let af = action_filter.to_ascii_lowercase();
        events.retain(|e| format!("{:?}", e.action).to_ascii_lowercase().contains(&af));
    }

    // Keep newest `limit` events (events are ordered ASC by created_at)
    if events.len() > limit {
        events = events.into_iter().rev().take(limit).rev().collect();
    }

    match serde_json::to_string_pretty(&events) {
        Ok(json) => json,
        Err(e) => format!("Error serializing events: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_hours() {
        let before = Utc::now() - Duration::hours(2);
        let result = parse_since("2h").unwrap();
        assert!((result - before).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_since_days() {
        let before = Utc::now() - Duration::days(3);
        let result = parse_since("3d").unwrap();
        assert!((result - before).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_since_iso8601() {
        let result = parse_since("2024-06-01T12:00:00Z").unwrap();
        assert_eq!(result.timestamp(), 1717243200);
    }

    #[test]
    fn parse_since_invalid_fails() {
        assert!(parse_since("notvalid").is_err());
        assert!(parse_since("Xh").is_err());
    }

    #[test]
    fn limit_capped_at_max() {
        // Verify the cap constant is sane
        assert!(MAX_LIMIT <= 200);
        assert!(DEFAULT_LIMIT <= MAX_LIMIT);
    }
}
