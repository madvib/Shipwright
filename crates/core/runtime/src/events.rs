use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::Row;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const EVENTS_FILE_NAME: &str = "events.ndjson";
const EVENT_INDEX_FILE: &str = "generated/event_index.json";
const TRACKED_DIRS: &[&str] = &[
    "project/specs",
    "project/features",
    "project/releases",
    "project/notes",
    "project/adrs",
];
const TRACKED_FILES: &[&str] = &[crate::config::PRIMARY_CONFIG_FILE];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum EventEntity {
    Project,
    Workspace,
    Session,
    Note,
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

impl EventEntity {
    fn as_db(&self) -> &'static str {
        match self {
            EventEntity::Project => "project",
            EventEntity::Workspace => "workspace",
            EventEntity::Session => "session",
            EventEntity::Note => "note",
            EventEntity::Spec => "spec",
            EventEntity::Adr => "adr",
            EventEntity::Feature => "feature",
            EventEntity::Release => "release",
            EventEntity::Config => "config",
            EventEntity::Mode => "mode",
            EventEntity::Prompt => "prompt",
            EventEntity::Plugin => "plugin",
            EventEntity::Ghost => "ghost",
            EventEntity::Time => "time",
            EventEntity::Agent => "agent",
            EventEntity::Mcp => "mcp",
        }
    }

    fn from_db(value: &str) -> Result<Self> {
        match value {
            "project" => Ok(EventEntity::Project),
            "workspace" => Ok(EventEntity::Workspace),
            "session" => Ok(EventEntity::Session),
            "note" => Ok(EventEntity::Note),
            "spec" => Ok(EventEntity::Spec),
            "adr" => Ok(EventEntity::Adr),
            "feature" => Ok(EventEntity::Feature),
            "release" => Ok(EventEntity::Release),
            "config" => Ok(EventEntity::Config),
            "mode" => Ok(EventEntity::Mode),
            "prompt" => Ok(EventEntity::Prompt),
            "plugin" => Ok(EventEntity::Plugin),
            "ghost" => Ok(EventEntity::Ghost),
            "time" => Ok(EventEntity::Time),
            "agent" => Ok(EventEntity::Agent),
            "mcp" => Ok(EventEntity::Mcp),
            other => Err(anyhow!("Unknown event entity '{}'", other)),
        }
    }
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

impl EventAction {
    fn as_db(&self) -> &'static str {
        match self {
            EventAction::Init => "init",
            EventAction::Create => "create",
            EventAction::Update => "update",
            EventAction::Delete => "delete",
            EventAction::Move => "move",
            EventAction::Note => "note",
            EventAction::Link => "link",
            EventAction::Add => "add",
            EventAction::Remove => "remove",
            EventAction::Set => "set",
            EventAction::Clear => "clear",
            EventAction::Scan => "scan",
            EventAction::Promote => "promote",
            EventAction::Start => "start",
            EventAction::Stop => "stop",
            EventAction::Log => "log",
        }
    }

    fn from_db(value: &str) -> Result<Self> {
        match value {
            "init" => Ok(EventAction::Init),
            "create" => Ok(EventAction::Create),
            "update" => Ok(EventAction::Update),
            "delete" => Ok(EventAction::Delete),
            "move" => Ok(EventAction::Move),
            "note" => Ok(EventAction::Note),
            "link" => Ok(EventAction::Link),
            "add" => Ok(EventAction::Add),
            "remove" => Ok(EventAction::Remove),
            "set" => Ok(EventAction::Set),
            "clear" => Ok(EventAction::Clear),
            "scan" => Ok(EventAction::Scan),
            "promote" => Ok(EventAction::Promote),
            "start" => Ok(EventAction::Start),
            "stop" => Ok(EventAction::Stop),
            "log" => Ok(EventAction::Log),
            other => Err(anyhow!("Unknown event action '{}'", other)),
        }
    }
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

fn event_index_path(project_dir: &Path) -> PathBuf {
    project_dir.join(EVENT_INDEX_FILE)
}

pub fn ensure_event_log(project_dir: &Path) -> Result<()> {
    let _conn = crate::state_db::open_project_connection(project_dir)?;
    Ok(())
}

fn row_to_event_record(row: &sqlx::sqlite::SqliteRow) -> Result<EventRecord> {
    let seq: i64 = row.get(0);
    let timestamp_raw: String = row.get(1);
    let actor: String = row.get(2);
    let entity_raw: String = row.get(3);
    let action_raw: String = row.get(4);
    let subject: String = row.get(5);
    let details: Option<String> = row.get(6);

    let parsed = DateTime::parse_from_rfc3339(&timestamp_raw)
        .with_context(|| format!("Invalid event timestamp '{}'", timestamp_raw))?;

    Ok(EventRecord {
        seq: seq as u64,
        timestamp: parsed.with_timezone(&Utc),
        actor,
        entity: EventEntity::from_db(&entity_raw)?,
        action: EventAction::from_db(&action_raw)?,
        subject,
        details,
    })
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
    let mut conn = crate::state_db::open_project_connection(project_dir)?;
    let timestamp = Utc::now();
    let timestamp_str = timestamp.to_rfc3339();
    let result = crate::state_db::block_on(async {
        sqlx::query(
            "INSERT INTO event_log (timestamp, actor, entity, action, subject, details)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&timestamp_str)
        .bind(actor)
        .bind(entity.as_db())
        .bind(action.as_db())
        .bind(&subject)
        .bind(&details)
        .execute(&mut conn)
        .await
    })?;
    let seq = result.last_insert_rowid() as u64;
    let record = EventRecord {
        seq,
        timestamp,
        actor: actor.to_string(),
        entity,
        action,
        subject,
        details,
    };

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
    ensure_event_log(project_dir)?;
    let mut conn = crate::state_db::open_project_connection(project_dir)?;
    let rows = crate::state_db::block_on(async {
        sqlx::query(
            "SELECT seq, timestamp, actor, entity, action, subject, details
             FROM event_log
             ORDER BY seq ASC",
        )
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_event_record).collect()
}

pub fn latest_event_seq(project_dir: &Path) -> Result<u64> {
    ensure_event_log(project_dir)?;
    let mut conn = crate::state_db::open_project_connection(project_dir)?;
    let row = crate::state_db::block_on(async {
        sqlx::query("SELECT COALESCE(MAX(seq), 0) FROM event_log")
            .fetch_one(&mut conn)
            .await
    })?;
    let seq: i64 = row.get(0);
    Ok(seq as u64)
}

pub fn list_events_since(
    project_dir: &Path,
    since_seq: u64,
    limit: Option<usize>,
) -> Result<Vec<EventRecord>> {
    ensure_event_log(project_dir)?;
    let mut conn = crate::state_db::open_project_connection(project_dir)?;

    let rows = if let Some(limit) = limit {
        crate::state_db::block_on(async {
            sqlx::query(
                "SELECT seq, timestamp, actor, entity, action, subject, details
                 FROM event_log
                 WHERE seq > ?
                 ORDER BY seq DESC
                 LIMIT ?",
            )
            .bind(since_seq as i64)
            .bind(limit as i64)
            .fetch_all(&mut conn)
            .await
        })?
    } else {
        crate::state_db::block_on(async {
            sqlx::query(
                "SELECT seq, timestamp, actor, entity, action, subject, details
                 FROM event_log
                 WHERE seq > ?
                 ORDER BY seq ASC",
            )
            .bind(since_seq as i64)
            .fetch_all(&mut conn)
            .await
        })?
    };

    let mut events: Vec<EventRecord> = rows
        .iter()
        .map(row_to_event_record)
        .collect::<Result<_>>()?;
    if limit.is_some() {
        events.reverse();
    }
    Ok(events)
}

pub fn export_events_ndjson(project_dir: &Path, output_path: &Path) -> Result<usize> {
    let events = read_events(project_dir)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = String::new();
    for event in &events {
        output.push_str(&serde_json::to_string(event)?);
        output.push('\n');
    }
    crate::fs_util::write_atomic(output_path, output)?;
    Ok(events.len())
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
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if file_name == "TEMPLATE.md" || file_name == "README.md" {
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
    if let Some(file) = rel_path.strip_prefix("project/specs/") {
        return Some((
            EventEntity::Spec,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("project/adrs/") {
        return Some((
            EventEntity::Adr,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("project/notes/") {
        return Some((
            EventEntity::Note,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("project/features/") {
        return Some((
            EventEntity::Feature,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if let Some(file) = rel_path.strip_prefix("project/releases/") {
        return Some((
            EventEntity::Release,
            file.to_string(),
            Some(format!("path={}", rel_path)),
        ));
    }
    if rel_path == crate::config::PRIMARY_CONFIG_FILE
        || rel_path == crate::config::LEGACY_CONFIG_FILE
    {
        return Some((
            EventEntity::Config,
            rel_path.to_string(),
            Some(format!("path={}", rel_path)),
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
