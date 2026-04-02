//! Terminal launcher — detects the host terminal environment and opens a tab
//! attaching to the given tmux session.
//!
//! Detection order (first match wins):
//!   `SHIP_DEFAULT_TERMINAL` env var overrides all auto-detection.
//!   auto-detect: wt.exe in PATH → $TMUX → $TERM_PROGRAM → uname → $DISPLAY/$WAYLAND_DISPLAY

/// The terminal strategy resolved for this host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalStrategy {
    Wt,
    Tmux,
    ITerm,
    MacTerminal,
    XdgTerminal(String),
    Manual,
}

impl TerminalStrategy {
    pub fn as_str(&self) -> &str {
        match self {
            TerminalStrategy::Wt => "wt",
            TerminalStrategy::Tmux => "tmux",
            TerminalStrategy::ITerm => "iterm",
            TerminalStrategy::MacTerminal => "terminal",
            TerminalStrategy::XdgTerminal(_) => "xdg",
            TerminalStrategy::Manual => "manual",
        }
    }
}

/// Build the command (program, args) to open a terminal tab for `session_name`.
/// Returns `None` for the manual strategy (no command to run).
pub fn build_command(strategy: &TerminalStrategy, session_name: &str) -> Option<(String, Vec<String>)> {
    match strategy {
        TerminalStrategy::Wt => Some((
            "wt.exe".to_string(),
            vec![
                "new-tab".to_string(),
                "--title".to_string(),
                session_name.to_string(),
                "--".to_string(),
                "wsl.exe".to_string(),
                "tmux".to_string(),
                "attach-session".to_string(),
                "-t".to_string(),
                session_name.to_string(),
            ],
        )),
        TerminalStrategy::Tmux => Some((
            "tmux".to_string(),
            vec![
                "new-window".to_string(),
                "-n".to_string(),
                session_name.to_string(),
                format!("tmux attach-session -t {session_name}"),
            ],
        )),
        TerminalStrategy::ITerm => {
            let script = format!(
                "tell application \"iTerm2\"\n\
                 tell current window\n\
                 create tab with default profile\n\
                 tell current session\n\
                 write text \"tmux attach-session -t {session_name}\"\n\
                 end tell\nend tell\nend tell"
            );
            Some(("osascript".to_string(), vec!["-e".to_string(), script]))
        }
        TerminalStrategy::MacTerminal => {
            let script = format!(
                "tell application \"Terminal\"\n\
                 do script \"tmux attach-session -t {session_name}\"\n\
                 activate\nend tell"
            );
            Some(("osascript".to_string(), vec!["-e".to_string(), script]))
        }
        TerminalStrategy::XdgTerminal(term) => Some((
            term.clone(),
            vec!["-e".to_string(), format!("tmux attach-session -t {session_name}")],
        )),
        TerminalStrategy::Manual => None,
    }
}

/// Detect the terminal strategy from the current environment.
pub fn detect_strategy() -> TerminalStrategy {
    detect_strategy_inner(
        std::env::var("SHIP_DEFAULT_TERMINAL").ok().as_deref().map(str::trim).map(str::to_lowercase).as_deref(),
        wt_in_path(),
        std::env::var("TMUX").is_ok(),
        std::env::var("TERM_PROGRAM").ok().as_deref(),
        std::env::consts::OS == "macos",
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok(),
        std::env::var("TERMINAL").ok().as_deref(),
    )
}

pub(crate) fn detect_strategy_inner(
    default_terminal: Option<&str>,
    wt_in_path: bool,
    tmux_env: bool,
    term_program: Option<&str>,
    is_darwin: bool,
    has_display: bool,
    terminal_env: Option<&str>,
) -> TerminalStrategy {
    if let Some(val) = default_terminal {
        match val {
            "wt" => return TerminalStrategy::Wt,
            "tmux" => return TerminalStrategy::Tmux,
            "manual" => return TerminalStrategy::Manual,
            _ => {}
        }
    }
    if wt_in_path {
        return TerminalStrategy::Wt;
    }
    if tmux_env {
        return TerminalStrategy::Tmux;
    }
    if term_program == Some("iTerm.app") {
        return TerminalStrategy::ITerm;
    }
    if is_darwin {
        return TerminalStrategy::MacTerminal;
    }
    if has_display {
        if let Some(term) = terminal_env.filter(|t| !t.is_empty()) {
            return TerminalStrategy::XdgTerminal(term.to_string());
        }
    }
    TerminalStrategy::Manual
}

/// Launch a terminal tab attaching to `session_name`.
///
/// Never hard-fails. Returns `(strategy_name, launched)`.
pub fn launch(session_name: &str) -> (String, bool) {
    let strategy = detect_strategy();
    let strategy_name = strategy.as_str().to_string();

    let Some((prog, args)) = build_command(&strategy, session_name) else {
        println!("ship: to attach, run: tmux attach-session -t {session_name}");
        return (strategy_name, false);
    };

    let launched = std::process::Command::new(&prog)
        .args(&args)
        .spawn()
        .is_ok();

    if !launched {
        tracing::warn!(
            session = session_name,
            strategy = strategy_name.as_str(),
            "terminal launch failed"
        );
    }

    (strategy_name, launched)
}

fn wt_in_path() -> bool {
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            if dir.join("wt.exe").exists() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wt_override_selects_wt_strategy() {
        let s = detect_strategy_inner(Some("wt"), false, false, None, false, false, None);
        assert_eq!(s, TerminalStrategy::Wt);
    }

    #[test]
    fn wt_command_contains_new_tab_and_wsl() {
        let (prog, args) = build_command(&TerminalStrategy::Wt, "my-session").unwrap();
        assert_eq!(prog, "wt.exe");
        assert!(args.contains(&"new-tab".to_string()));
        assert!(args.contains(&"wsl.exe".to_string()));
        assert!(args.contains(&"my-session".to_string()));
    }

    #[test]
    fn tmux_override_selects_tmux_strategy() {
        let s = detect_strategy_inner(Some("tmux"), false, false, None, false, false, None);
        assert_eq!(s, TerminalStrategy::Tmux);
    }

    #[test]
    fn tmux_command_uses_new_window() {
        let (prog, args) = build_command(&TerminalStrategy::Tmux, "my-session").unwrap();
        assert_eq!(prog, "tmux");
        assert!(args.contains(&"new-window".to_string()));
    }

    #[test]
    fn manual_override_returns_no_command() {
        let s = detect_strategy_inner(Some("manual"), false, false, None, false, false, None);
        assert_eq!(s, TerminalStrategy::Manual);
        assert!(build_command(&s, "my-session").is_none());
    }

    #[test]
    fn auto_wt_in_path_wins_over_tmux_env() {
        let s = detect_strategy_inner(None, true, true, None, false, false, None);
        assert_eq!(s, TerminalStrategy::Wt);
    }

    #[test]
    fn auto_tmux_env_when_no_wt() {
        let s = detect_strategy_inner(None, false, true, None, false, false, None);
        assert_eq!(s, TerminalStrategy::Tmux);
    }

    #[test]
    fn auto_iterm_when_term_program_set() {
        let s = detect_strategy_inner(None, false, false, Some("iTerm.app"), false, false, None);
        assert_eq!(s, TerminalStrategy::ITerm);
    }

    #[test]
    fn auto_darwin_fallback() {
        let s = detect_strategy_inner(None, false, false, None, true, false, None);
        assert_eq!(s, TerminalStrategy::MacTerminal);
    }

    #[test]
    fn auto_xdg_when_display_and_terminal_set() {
        let s = detect_strategy_inner(None, false, false, None, false, true, Some("xterm"));
        assert_eq!(s, TerminalStrategy::XdgTerminal("xterm".to_string()));
    }

    #[test]
    fn headless_falls_back_to_manual() {
        let s = detect_strategy_inner(None, false, false, None, false, false, None);
        assert_eq!(s, TerminalStrategy::Manual);
    }

    #[test]
    fn display_set_but_no_terminal_env_falls_back_to_manual() {
        let s = detect_strategy_inner(None, false, false, None, false, true, None);
        assert_eq!(s, TerminalStrategy::Manual);
    }
}
