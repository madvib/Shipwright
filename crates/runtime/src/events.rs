use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const EVENTS_FILE_NAME: &str = "events.ndjson";
const EVENT_INDEX_FILE: &str = "workflow/event_index.json";
const TRACKED_DIRS: &[&str] = &["issues", "specs", "adrs", "features", "releases"];
const TRACKED_FILES: &[&str] = &["config.toml"];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum EventEntity {
    Project,
    Issue,
    Spec,
    Adr,
    Feature,
    Release,
    Config,
    Mode,
    Prompt,
    Plugin,
    Ghost,
    Time,
    Agent,
    Mcp,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum EventAction {
    Init,
    Create,
    Update,
    Delete,
    Move,
    Note,
    Link,
    Add,
    Remove,
    Set,
    Clear,
    Scan,
    Promote,
    Start,
    Stop,
    Log,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct EventRecord {
    pub seq: u64,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub entity: EventEntity,
    pub action: EventAction,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct FileFingerprint {
    modified_nanos: u128,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct EventSnapshot {
    files: HashMap<String, FileFingerprint>,
}

pub fn event_log_path(project_dir: &Path) -> PathBuf {
    project_dir.join(EVENTS_FILE_NAME)
}

fn event_index_path(project_dir: &Path) -> PathBuf {
    project_dir.join(EVENT_INDEX_FILE)
}

pub fn ensure_event_log(project_dir: &Path) -> Result<()> {
    let path = event_log_path(project_dir);
    if !path.exists() {
        fs::write(path, "")?;
    }
    Ok(())
}

fn read_events_from_path(path: &Path) -> Result<Vec<EventRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read event log: {}", path.display()))?;
    let mut events = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: EventRecord = serde_json::from_str(line).map_err(|e| {
            anyhow!(
                "Failed to parse event log {} line {}: {}",
                path.display(),
                idx + 1,
                e
            )
        })?;
        events.push(event);
    }
    Ok(events)
}

fn append_event_internal(
    project_dir: &Path,
    actor: &str,
    entity: EventEntity,
    action: EventAction,
    subject: String,
    details: Option<String>,
    sync_snapshot: bool,
) -> Result<EventRecord> {
    ensure_event_log(project_dir)?;
    let path = event_log_path(project_dir);
    let seq = latest_event_seq(project_dir)? + 1;
    let record = EventRecord {
        seq,
        timestamp: Utc::now(),
        actor: actor.to_string(),
        entity,
        action,
        subject,
        details,
    };

    let line = serde_json::to_string(&record).context("Failed to serialise event")?;
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .with_context(|| format!("Failed to open event log: {}", path.display()))?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;

    if sync_snapshot {
        let _ = sync_event_snapshot(project_dir)?;
    }
    Ok(record)
}

pub fn append_event(
    project_dir: &Path,
    actor: &str,
    entity: EventEntity,
    action: EventAction,
    subject: impl Into<String>,
    details: Option<String>,
) -> Result<EventRecord> {
    append_event_internal(
        project_dir,
        actor,
        entity,
        action,
        subject.into(),
        details,
        true,
    )
}

pub fn read_events(project_dir: &Path) -> Result<Vec<EventRecord>> {
    read_events_from_path(&event_log_path(project_dir))
}

pub fn latest_event_seq(project_dir: &Path) -> Result<u64> {
    Ok(read_events(project_dir)?.last().map(|e| e.seq).unwrap_or(0))
}

pub fn list_events_since(
    project_dir: &Path,
    since_seq: u64,
    limit: Option<usize>,
) -> Result<Vec<EventRecord>> {
    let mut events: Vec<EventRecord> = read_events(project_dir)?
        .into_iter()
        .filter(|e| e.seq > since_seq)
        .collect();
    if let Some(limit) = limit {
        if events.len() > limit {
            events = events[events.len() - limit..].to_vec();
        }
    }
    Ok(events)
}

fn load_snapshot(project_dir: &Path) -> Result<EventSnapshot> {
    let path = event_index_path(project_dir);
    if !path.exists() {
        return Ok(EventSnapshot::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read event index: {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(EventSnapshot::default());
    }
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

fn save_snapshot(project_dir: &Path, snapshot: &EventSnapshot) -> Result<()> {
    let path = event_index_path(project_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(snapshot)?;
    fs::write(path, json)?;
    Ok(())
}

fn modified_nanos(path: &Path) -> Option<u128> {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
}

fn collect_tracked_files(project_dir: &Path) -> Result<HashMap<String, FileFingerprint>> {
    let mut files: HashMap<String, FileFingerprint> = HashMap::new();

    for dir in TRACKED_DIRS {
        let root = project_dir.join(dir);
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(&root) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if !path.is_file() || !path.extension().is_some_and(|ext| ext == "md") {
                continue;
            }
            let rel = path
                .strip_prefix(project_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let metadata = fs::metadata(path)
                .with_context(|| format!("Failed to stat tracked file: {}", path.display()))?;
            if let Some(modified) = modified_nanos(path) {
                files.insert(
                    rel,
                    FileFingerprint {
                        modified_nanos: modified,
                        size: metadata.len(),
                    },
                );
            }
        }
    }

    for file in TRACKED_FILES {
        let path = project_dir.join(file);
        if !path.exists() || !path.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(project_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = fs::metadata(&path)
            .with_context(|| format!("Failed to stat tracked file: {}", path.display()))?;
        if let Some(modified) = modified_nanos(&path) {
            files.insert(
                rel,
                FileFingerprint {
                    modified_nanos: modified,
                    size: metadata.len(),
                },
            );
        }
    }

    Ok(files)
}

pub fn sync_event_snapshot(project_dir: &Path) -> Result<usize> {
    let snapshot = EventSnapshot {
        files: collect_tracked_files(project_dir)?,
    };
    let count = snapshot.files.len();
    save_snapshot(project_dir, &snapshot)?;
    Ok(count)
}

fn classify_path(rel_path: &str) -> Option<(EventEntity, String, Option<String>)> {
    if let Some(rest) = rel_path.strip_prefix("issues/") {
        let mut parts = rest.splitn(2, '/');
        let status = parts.next().unwrap_or("").to_string();
        let file_name = parts.next().unwrap_or("").to_string();
        return Some((
            EventEntity::Issue,
            file_name.clone(),
            Some(format!("status={} path={}", status, rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("specs/") {
        return Some((
            EventEntity::Spec,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("adrs/") {
        return Some((
            EventEntity::Adr,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("features/") {
        return Some((
            EventEntity::Feature,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("releases/") {
        return Some((
            EventEntity::Release,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if rel_path == "config.toml" {
        return Some((
            EventEntity::Config,
            "config.toml".to_string(),
            Some("path=config.toml".to_string()),
        ));
    }
    None
}

fn merge_details(lhs: Option<String>, rhs: Option<String>) -> Option<String> {
    match (lhs, rhs) {
        (None, None) => None,
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (Some(a), Some(b)) => Some(format!("{} {}", a, b)),
    }
}

pub fn ingest_external_events(project_dir: &Path) -> Result<Vec<EventRecord>> {
    ensure_event_log(project_dir)?;
    let previous = load_snapshot(project_dir)?;
    let current_files = collect_tracked_files(project_dir)?;
    let current_snapshot = EventSnapshot {
        files: current_files.clone(),
    };

    let mut keys: BTreeSet<String> = BTreeSet::new();
    keys.extend(previous.files.keys().cloned());
    keys.extend(current_files.keys().cloned());

    let mut emitted = Vec::new();
    for key in keys {
        let prev = previous.files.get(&key);
        let curr = current_files.get(&key);

        let action = match (prev, curr) {
            (None, Some(_)) => Some(EventAction::Create),
            (Some(_), None) => Some(EventAction::Delete),
            (Some(a), Some(b)) if a != b => Some(EventAction::Update),
            _ => None,
        };
        let Some(action) = action else {
            continue;
        };
        let Some((entity, subject, base_details)) = classify_path(&key) else {
            continue;
        };

        let details = match action {
            EventAction::Update => {
                let delta = match (prev, curr) {
                    (Some(a), Some(b)) => Some(format!(
                        "size={}→{} mtime={}→{}",
                        a.size, b.size, a.modified_nanos, b.modified_nanos
                    )),
                    _ => None,
                };
                merge_details(base_details, delta)
            }
            _ => base_details,
        };

        let record = append_event_internal(
            project_dir,
            "filesystem",
            entity,
            action,
            subject,
            details,
            false,
        )?;
        emitted.push(record);
    }

    save_snapshot(project_dir, &current_snapshot)?;
    Ok(emitted)
}
