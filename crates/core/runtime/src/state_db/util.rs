use anyhow::{Result, anyhow};
use sqlx::SqliteConnection;
use std::path::Path;

pub fn block_on<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    // If we are inside a Tokio runtime (e.g. spawn_blocking thread or MCP
    // async handler), use block_in_place to run the future without blocking
    // the scheduler thread. block_in_place is safe to call from any thread
    // that is within the Tokio threadpool, including blocking threads.
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| {
            handle
                .block_on(future)
                .map_err(|err| anyhow!("SQLite operation failed: {}", err))
        });
    }
    // No runtime active — create a lightweight single-threaded one.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|err| anyhow!("Failed to create SQLite runtime: {}", err))?;
    runtime
        .block_on(future)
        .map_err(|err| anyhow!("SQLite operation failed: {}", err))
}

pub fn table_exists(connection: &mut SqliteConnection, table: &str) -> Result<bool> {
    let row = block_on(async {
        sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(table)
            .fetch_optional(&mut *connection)
            .await
    })?;
    Ok(row.is_some())
}

pub fn column_exists(connection: &mut SqliteConnection, table: &str, column: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({})", table);
    let rows = block_on(async { sqlx::query(&pragma).fetch_all(&mut *connection).await })?;
    use sqlx::Row;
    Ok(rows
        .iter()
        .any(|row| row.get::<String, _>(1).eq_ignore_ascii_case(column)))
}

pub fn sqlite_url(path: &Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{}", raw);
    }
    format!("sqlite://{}", raw)
}
