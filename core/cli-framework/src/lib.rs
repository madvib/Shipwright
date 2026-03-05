use anyhow::{Context, Result, bail};
use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod core_primitives;

pub use core_primitives::{
    EventAction, McpAction, ModeAction, ProviderAction, SkillAction, SkillReadScope,
    SkillWriteScope, handle_event_action, handle_mcp_action, handle_mode_action,
    handle_provider_action, handle_skill_action, parse_skill_read_scope, parse_skill_write_scope,
};

#[derive(Debug, Clone, Copy)]
pub struct CliMetadata {
    pub id: &'static str,
    pub display_name: &'static str,
    pub version: &'static str,
}

impl CliMetadata {
    pub const fn new(id: &'static str, display_name: &'static str, version: &'static str) -> Self {
        Self {
            id,
            display_name,
            version,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CliRunContext {
    started_at: Instant,
}

impl CliRunContext {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

#[derive(Debug, Clone)]
pub enum CoreCommand {
    Init { path: Option<PathBuf> },
    Doctor,
    Version,
}

#[derive(Debug, Clone)]
pub struct InitTarget {
    pub path: PathBuf,
    pub project_name: String,
}

impl InitTarget {
    pub fn new(path: PathBuf, project_name: String) -> Self {
        Self { path, project_name }
    }
}

#[derive(Debug, Clone)]
pub struct VersionDetail {
    pub key: &'static str,
    pub value: String,
}

impl VersionDetail {
    pub fn new(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DoctorSeverity {
    Ok,
    Warn,
    Fail,
}

impl DoctorSeverity {
    fn symbol(self) -> &'static str {
        match self {
            Self::Ok => "✓",
            Self::Warn => "⚠",
            Self::Fail => "✗",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoctorCheck {
    pub severity: DoctorSeverity,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Default, Clone)]
pub struct DoctorReport {
    checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ok(&mut self, label: impl Into<String>, detail: impl Into<String>) {
        self.push(DoctorSeverity::Ok, label, detail);
    }

    pub fn warn(&mut self, label: impl Into<String>, detail: impl Into<String>) {
        self.push(DoctorSeverity::Warn, label, detail);
    }

    pub fn fail(&mut self, label: impl Into<String>, detail: impl Into<String>) {
        self.push(DoctorSeverity::Fail, label, detail);
    }

    pub fn checks(&self) -> &[DoctorCheck] {
        &self.checks
    }

    pub fn print(&self) {
        for check in &self.checks {
            println!(
                "{} {}: {}",
                check.severity.symbol(),
                check.label,
                check.detail
            );
        }
    }

    fn push(
        &mut self,
        severity: DoctorSeverity,
        label: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.checks.push(DoctorCheck {
            severity,
            label: label.into(),
            detail: detail.into(),
        });
    }
}

pub trait CliApp {
    type Cli: Parser;
    fn metadata() -> CliMetadata;

    fn preflight(_context: &CliRunContext) -> Result<()> {
        Ok(())
    }

    fn classify_core_command(_cli: &Self::Cli) -> Option<CoreCommand> {
        None
    }

    fn init(_target: InitTarget, _context: &CliRunContext) -> Result<()> {
        bail!("init command not implemented for this CLI app")
    }

    fn doctor(_report: &mut DoctorReport, _context: &CliRunContext) -> Result<()> {
        Ok(())
    }

    fn version_details() -> Vec<VersionDetail> {
        Vec::new()
    }

    fn handle(cli: Self::Cli, context: &CliRunContext) -> Result<()>;

    fn postflight(_context: &CliRunContext) -> Result<()> {
        Ok(())
    }
}

pub fn run<A: CliApp>() -> Result<()> {
    run_with_args::<A, _, _>(std::env::args_os())
}

pub fn run_with_args<A, I, T>(args: I) -> Result<()>
where
    A: CliApp,
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let metadata = A::metadata();
    let context = CliRunContext::new();

    A::preflight(&context)
        .with_context(|| format!("{} preflight failed", metadata.display_name))?;
    let cli = <A::Cli as Parser>::parse_from(args);
    if let Some(core_command) = A::classify_core_command(&cli) {
        handle_core_command::<A>(core_command, &context)
            .with_context(|| format!("{} execution failed", metadata.display_name))?;
    } else {
        A::handle(cli, &context)
            .with_context(|| format!("{} execution failed", metadata.display_name))?;
    }
    A::postflight(&context)
        .with_context(|| format!("{} postflight failed", metadata.display_name))?;

    Ok(())
}

fn handle_core_command<A: CliApp>(
    core_command: CoreCommand,
    context: &CliRunContext,
) -> Result<()> {
    match core_command {
        CoreCommand::Init { path } => {
            let path = resolve_init_target_path(path)?;
            let project_name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| "project".to_string());
            A::init(InitTarget::new(path, project_name), context)
        }
        CoreCommand::Doctor => {
            let metadata = A::metadata();
            println!(
                "Checking {} environment (v{})...",
                metadata.display_name, metadata.version
            );

            let mut report = DoctorReport::new();
            report.ok(
                "CLI framework",
                format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            );
            match std::env::current_exe() {
                Ok(path) => report.ok("Current executable", path.display().to_string()),
                Err(err) => report.fail("Current executable", err.to_string()),
            }
            A::doctor(&mut report, context)?;
            report.print();
            Ok(())
        }
        CoreCommand::Version => {
            let metadata = A::metadata();
            println!("{} version {}", metadata.id, metadata.version);
            for detail in A::version_details() {
                println!("{} {}", detail.key, detail.value);
            }
            Ok(())
        }
    }
}

fn resolve_init_target_path(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(path) => {
            if let Ok(canonical) = std::fs::canonicalize(&path) {
                return Ok(canonical);
            }
            if path.is_absolute() {
                Ok(path)
            } else {
                Ok(std::env::current_dir()?.join(path))
            }
        }
        None => Ok(std::env::current_dir()?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Subcommand;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU8, Ordering};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    static STAGE: AtomicU8 = AtomicU8::new(0);

    #[derive(Parser)]
    struct TestCli {
        #[arg(long)]
        name: String,
    }

    struct TestApp;

    impl CliApp for TestApp {
        type Cli = TestCli;

        fn metadata() -> CliMetadata {
            CliMetadata::new("test-cli", "Test CLI", "0.0.0")
        }

        fn preflight(context: &CliRunContext) -> Result<()> {
            assert_eq!(STAGE.load(Ordering::SeqCst), 0);
            assert!(context.elapsed() >= Duration::ZERO);
            STAGE.store(1, Ordering::SeqCst);
            Ok(())
        }

        fn handle(cli: Self::Cli, context: &CliRunContext) -> Result<()> {
            assert_eq!(STAGE.load(Ordering::SeqCst), 1);
            assert_eq!(cli.name, "ship");
            assert!(context.elapsed() >= Duration::ZERO);
            STAGE.store(2, Ordering::SeqCst);
            Ok(())
        }

        fn postflight(context: &CliRunContext) -> Result<()> {
            assert_eq!(STAGE.load(Ordering::SeqCst), 2);
            assert!(context.elapsed() >= Duration::ZERO);
            STAGE.store(3, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn run_with_args_executes_lifecycle_hooks() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        STAGE.store(0, Ordering::SeqCst);

        run_with_args::<TestApp, _, _>(["ship", "--name", "ship"]).expect("run test app");

        assert_eq!(STAGE.load(Ordering::SeqCst), 3);
    }

    static CORE_STAGE: AtomicU8 = AtomicU8::new(0);

    #[derive(Parser, Debug)]
    struct CoreCli {
        #[command(subcommand)]
        command: Option<CoreCliCommands>,
    }

    #[derive(Subcommand, Debug)]
    enum CoreCliCommands {
        Init { path: Option<PathBuf> },
        Doctor,
        Version,
        Run,
    }

    struct CoreTestApp;

    impl CliApp for CoreTestApp {
        type Cli = CoreCli;

        fn metadata() -> CliMetadata {
            CliMetadata::new("ship", "Ship", "9.9.9")
        }

        fn classify_core_command(cli: &Self::Cli) -> Option<CoreCommand> {
            match cli.command.as_ref() {
                Some(CoreCliCommands::Init { path }) => {
                    Some(CoreCommand::Init { path: path.clone() })
                }
                Some(CoreCliCommands::Doctor) => Some(CoreCommand::Doctor),
                Some(CoreCliCommands::Version) => Some(CoreCommand::Version),
                Some(CoreCliCommands::Run) | None => None,
            }
        }

        fn init(target: InitTarget, _context: &CliRunContext) -> Result<()> {
            assert_eq!(target.project_name, "test-project");
            CORE_STAGE.store(1, Ordering::SeqCst);
            Ok(())
        }

        fn doctor(report: &mut DoctorReport, _context: &CliRunContext) -> Result<()> {
            assert!(
                report
                    .checks()
                    .iter()
                    .any(|check| check.label == "CLI framework"),
                "framework doctor report should include framework version check"
            );
            report.warn("doctor-hook", "ran");
            CORE_STAGE.store(2, Ordering::SeqCst);
            Ok(())
        }

        fn version_details() -> Vec<VersionDetail> {
            CORE_STAGE.store(3, Ordering::SeqCst);
            vec![VersionDetail::new("built_at", "now")]
        }

        fn handle(_cli: Self::Cli, _context: &CliRunContext) -> Result<()> {
            CORE_STAGE.store(4, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn run_with_args_routes_init_to_framework_hook() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        CORE_STAGE.store(0, Ordering::SeqCst);
        run_with_args::<CoreTestApp, _, _>(["ship", "init", "test-project"]).expect("init");
        assert_eq!(CORE_STAGE.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn run_with_args_routes_doctor_to_framework_hook() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        CORE_STAGE.store(0, Ordering::SeqCst);
        run_with_args::<CoreTestApp, _, _>(["ship", "doctor"]).expect("doctor");
        assert_eq!(CORE_STAGE.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn run_with_args_routes_version_to_framework_hook() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        CORE_STAGE.store(0, Ordering::SeqCst);
        run_with_args::<CoreTestApp, _, _>(["ship", "version"]).expect("version");
        assert_eq!(CORE_STAGE.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn run_with_args_routes_non_core_to_app_handler() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        CORE_STAGE.store(0, Ordering::SeqCst);
        run_with_args::<CoreTestApp, _, _>(["ship", "run"]).expect("run");
        assert_eq!(CORE_STAGE.load(Ordering::SeqCst), 4);
    }
}
