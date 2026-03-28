use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use std::path::Path;

/// Parse a relative time string ("1h", "24h", "7d") or ISO 8601 timestamp
/// into an absolute UTC cutoff. Events before this time are excluded.
fn parse_since(raw: &str) -> Result<DateTime<Utc>> {
    let s = raw.trim();

    // Try relative formats first: Nh, Nd
    if let Some(h) = s.strip_suffix('h') {
        let hours: i64 = h.parse().map_err(|_| anyhow!("Invalid hours: '{}'", s))?;
        return Ok(Utc::now() - Duration::hours(hours));
    }
    if let Some(d) = s.strip_suffix('d') {
        let days: i64 = d.parse().map_err(|_| anyhow!("Invalid days: '{}'", s))?;
        return Ok(Utc::now() - Duration::days(days));
    }

    // Fall back to ISO 8601 parse
    s.parse::<DateTime<Utc>>().or_else(|_| {
        chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| anyhow!("Could not parse '{}' as a time: {}", s, e))
    })
}

/// Truncate a string to at most `max_chars` characters for table display.
fn trunc(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        chars[..max_chars - 1].iter().collect::<String>() + "…"
    }
}

/// Format a UTC timestamp as a compact local-ish string for table output.
fn fmt_ts(ts: &DateTime<Utc>) -> String {
    ts.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn run_events(
    ship_dir: &Path,
    since: Option<String>,
    actor: Option<String>,
    entity: Option<String>,
    action: Option<String>,
    limit: u32,
    json: bool,
) -> Result<()> {
    let since_cutoff = since.as_deref().map(parse_since).transpose()?;

    let mut events = runtime::events::read_events(ship_dir)?;

    // Apply filters
    if let Some(cutoff) = since_cutoff {
        events.retain(|e| e.created_at >= cutoff);
    }
    if let Some(ref actor_filter) = actor {
        events.retain(|e| e.actor.contains(actor_filter.as_str()));
    }
    if let Some(ref entity_filter) = entity {
        let ef = entity_filter.to_ascii_lowercase();
        events.retain(|e| e.event_type.to_ascii_lowercase().contains(&ef));
    }
    if let Some(ref action_filter) = action {
        let af = action_filter.to_ascii_lowercase();
        events.retain(|e| e.event_type.to_ascii_lowercase().contains(&af));
    }

    // Keep newest N events (take from the end since events are ordered ASC)
    let limit_usize = limit as usize;
    if events.len() > limit_usize {
        events = events.into_iter().rev().take(limit_usize).rev().collect();
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&events)?);
        return Ok(());
    }

    if events.is_empty() {
        println!("No events found.");
        return Ok(());
    }

    // Table header
    println!(
        "{:<10} {:<20} {:<16} {:<28} ENTITY",
        "ID", "TIMESTAMP", "ACTOR", "EVENT_TYPE"
    );
    println!("{}", "-".repeat(90));

    for ev in &events {
        println!(
            "{:<10} {:<20} {:<16} {:<28} {}",
            trunc(&ev.id, 10),
            fmt_ts(&ev.created_at),
            trunc(&ev.actor, 16),
            trunc(&ev.event_type, 28),
            trunc(&ev.entity_id, 36),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn parse_relative_hours() {
        let before = Utc::now() - Duration::hours(1);
        let result = parse_since("1h").unwrap();
        // Should be within 2s of now-1h
        assert!((result - before).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_relative_days() {
        let before = Utc::now() - Duration::days(7);
        let result = parse_since("7d").unwrap();
        assert!((result - before).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_iso8601() {
        let result = parse_since("2025-01-01T00:00:00Z").unwrap();
        assert_eq!(result.year(), 2025);
    }

    #[test]
    fn parse_invalid_returns_err() {
        assert!(parse_since("notadate").is_err());
    }

    #[test]
    fn trunc_short_string_unchanged() {
        assert_eq!(trunc("hello", 10), "hello");
    }

    #[test]
    fn trunc_long_string_truncated() {
        let result = trunc("abcdefghij", 5);
        assert_eq!(result.chars().count(), 5);
        assert!(result.ends_with('…'));
    }
}
