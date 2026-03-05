use anyhow::{Context, Result};
use async_trait::async_trait;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct McpMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub version: &'static str,
}

impl McpMetadata {
    pub const fn new(id: &'static str, display_name: &'static str, version: &'static str) -> Self {
        Self {
            id,
            display_name,
            version,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct McpRunContext {
    started_at: Instant,
}

impl McpRunContext {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

#[async_trait]
pub trait McpApp {
    fn metadata() -> McpMetadata;

    fn startup_banner(metadata: McpMetadata) -> Option<String> {
        Some(format!(
            "{} MCP server v{} starting...",
            metadata.display_name, metadata.version
        ))
    }

    fn shutdown_banner(metadata: McpMetadata, context: &McpRunContext) -> Option<String> {
        Some(format!(
            "{} MCP server stopped after {:.2?}",
            metadata.display_name,
            context.elapsed()
        ))
    }

    async fn preflight(_context: &McpRunContext) -> Result<()> {
        Ok(())
    }

    async fn serve(context: &McpRunContext) -> Result<()>;

    async fn postflight(_context: &McpRunContext) -> Result<()> {
        Ok(())
    }
}

pub async fn run<A: McpApp>() -> Result<()> {
    let metadata = A::metadata();
    let context = McpRunContext::new();
    if let Some(startup) = A::startup_banner(metadata) {
        eprintln!("{}", startup);
    }

    A::preflight(&context)
        .await
        .with_context(|| format!("{} preflight failed", metadata.display_name))?;
    A::serve(&context)
        .await
        .with_context(|| format!("{} execution failed", metadata.display_name))?;
    A::postflight(&context)
        .await
        .with_context(|| format!("{} postflight failed", metadata.display_name))?;
    if let Some(shutdown) = A::shutdown_banner(metadata, &context) {
        eprintln!("{}", shutdown);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    static STAGE: AtomicU8 = AtomicU8::new(0);

    struct TestApp;

    #[async_trait]
    impl McpApp for TestApp {
        fn metadata() -> McpMetadata {
            McpMetadata::new("test-mcp", "Test MCP", "0.0.0")
        }

        async fn preflight(context: &McpRunContext) -> Result<()> {
            assert_eq!(STAGE.load(Ordering::SeqCst), 0);
            assert!(context.elapsed() >= Duration::ZERO);
            STAGE.store(1, Ordering::SeqCst);
            Ok(())
        }

        async fn serve(context: &McpRunContext) -> Result<()> {
            assert_eq!(STAGE.load(Ordering::SeqCst), 1);
            assert!(context.elapsed() >= Duration::ZERO);
            STAGE.store(2, Ordering::SeqCst);
            Ok(())
        }

        async fn postflight(context: &McpRunContext) -> Result<()> {
            assert_eq!(STAGE.load(Ordering::SeqCst), 2);
            assert!(context.elapsed() >= Duration::ZERO);
            STAGE.store(3, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_executes_lifecycle_hooks() {
        STAGE.store(0, Ordering::SeqCst);

        run::<TestApp>().await.expect("run test app");

        assert_eq!(STAGE.load(Ordering::SeqCst), 3);
    }
}
