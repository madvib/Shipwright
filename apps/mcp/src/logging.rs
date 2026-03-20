use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize file + stderr logging for ship-mcp.
///
/// Log file: `~/.ship/logs/ship-mcp.log`
/// - INFO and above go to the file (always)
/// - WARN and above go to stderr (when stderr is visible)
///
/// When Claude Code manages ship-mcp as a stdio process, stderr is discarded.
/// File logging is the only reliable channel for capturing operational errors.
///
/// Returns the appender guard — keep it alive for the duration of the process.
pub fn init() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let logs_dir = dirs::home_dir()?.join(".ship").join("logs");
    if std::fs::create_dir_all(&logs_dir).is_err() {
        return None;
    }

    let file_appender = tracing_appender::rolling::never(&logs_dir, "ship-mcp.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let stderr_filter = EnvFilter::try_from_env("SHIP_MCP_LOG")
        .unwrap_or_else(|_| EnvFilter::new("warn"));
    let file_filter = EnvFilter::new("info");

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(stderr_filter);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_filter(file_filter);

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();

    Some(guard)
}
