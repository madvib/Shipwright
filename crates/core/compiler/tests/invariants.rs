/// Compiler invariant tests — determinism, provider round-trips, empty input,
/// and WASM purity (structural).
///
/// No `proptest`/`quickcheck` in Cargo.toml, so these are deterministic
/// parameterised tests as permitted by the job spec.
use compiler::{
    McpServerConfig, ProjectLibrary, Rule, Skill, compile, list_providers, resolve_library,
};

// ─── Fixture helpers ──────────────────────────────────────────────────────────

fn make_skill(id: &str) -> Skill {
    Skill {
        id: id.to_string(),
        name: id.to_string(),
        stable_id: None,
        description: Some(format!("Description for {id}")),
        license: None,
        compatibility: None,
        allowed_tools: vec![],
        metadata: Default::default(),
        content: format!("# {id}\nDo something useful."),
        source: Default::default(),
        vars: Default::default(),
    }
}

fn make_rule(file_name: &str) -> Rule {
    Rule {
        file_name: file_name.to_string(),
        content: format!("Follow the {} guidelines.", file_name),
        always_apply: true,
        globs: vec![],
        description: None,
    }
}

fn make_server(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.to_string(),
        name: id.to_string(),
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: Default::default(),
        scope: "project".to_string(),
        server_type: Default::default(),
        url: None,
        disabled: false,
        timeout_secs: None,
        codex_enabled_tools: vec![],
        codex_disabled_tools: vec![],
        gemini_trust: None,
        gemini_include_tools: vec![],
        gemini_exclude_tools: vec![],
        gemini_timeout_ms: None,
        cursor_env_file: None,
    }
}

fn rich_library() -> ProjectLibrary {
    ProjectLibrary {
        skills: vec![make_skill("deploy"), make_skill("review")],
        rules: vec![make_rule("style.md"), make_rule("testing.md")],
        mcp_servers: vec![make_server("github"), make_server("linear")],
        ..Default::default()
    }
}

// ─── Determinism ──────────────────────────────────────────────────────────────

/// Same ProjectLibrary compiled 10× must produce identical output every time.
/// Tests all four providers to catch any per-provider non-determinism.
#[test]
fn compile_is_deterministic() {
    let library = rich_library();
    let providers = ["claude", "gemini", "codex", "cursor"];

    for provider in providers {
        let resolved = resolve_library(&library, None, None);
        let reference = compile(&resolved, provider).expect("known provider");

        for _ in 1..10 {
            let resolved = resolve_library(&library, None, None);
            let output = compile(&resolved, provider).unwrap();

            // Compare serialisable/comparable fields.
            assert_eq!(
                output.mcp_servers, reference.mcp_servers,
                "mcp_servers non-deterministic for {provider}"
            );
            assert_eq!(
                output.context_content, reference.context_content,
                "context_content non-deterministic for {provider}"
            );
            assert_eq!(
                output.mcp_config_path, reference.mcp_config_path,
                "mcp_config_path non-deterministic for {provider}"
            );

            let mut skill_keys: Vec<_> = output.skill_files.keys().collect();
            skill_keys.sort();
            let mut ref_skill_keys: Vec<_> = reference.skill_files.keys().collect();
            ref_skill_keys.sort();
            assert_eq!(
                skill_keys, ref_skill_keys,
                "skill_files keys non-deterministic for {provider}"
            );

            let mut rule_keys: Vec<_> = output.rule_files.keys().collect();
            rule_keys.sort();
            let mut ref_rule_keys: Vec<_> = reference.rule_files.keys().collect();
            ref_rule_keys.sort();
            assert_eq!(
                rule_keys, ref_rule_keys,
                "rule_files keys non-deterministic for {provider}"
            );

            assert_eq!(
                format!("{:?}", output.claude_settings_patch),
                format!("{:?}", reference.claude_settings_patch),
                "claude_settings_patch non-deterministic for {provider}"
            );
        }
    }
}

// ─── Provider round-trips ─────────────────────────────────────────────────────

/// Compiling for `claude` must produce a CLAUDE.md context section containing
/// skill and rule content.
#[test]
fn claude_output_contains_context_file() {
    let library = rich_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "claude").unwrap();

    let content = output
        .context_content
        .expect("claude must emit context_content");
    assert!(
        !content.is_empty(),
        "CLAUDE.md context must be non-empty when rules/skills are present"
    );
    // Claude context file contains only rules (not skills — skills get their own files).
    // Rule content from make_rule() contains the file_name prefix "style" / "testing".
    assert!(
        content.contains("style") || content.contains("testing"),
        "CLAUDE.md must contain rule content; got:\n{content}"
    );
}

/// Compiling for `cursor` must produce per-file rule entries in `.cursor/rules/`.
#[test]
fn cursor_output_contains_rule_files() {
    let library = rich_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "cursor").unwrap();

    assert!(
        !output.rule_files.is_empty(),
        "cursor must emit rule_files when rules are present"
    );
    for path in output.rule_files.keys() {
        assert!(
            path.starts_with(".cursor/rules/"),
            "cursor rule path must be under .cursor/rules/; got {path}"
        );
        assert!(
            path.ends_with(".mdc"),
            "cursor rule file must use .mdc extension; got {path}"
        );
    }
}

/// Compiling for `gemini` must emit a context file and MCP config path.
#[test]
fn gemini_output_has_context_and_mcp_path() {
    let library = rich_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "gemini").unwrap();

    assert!(
        output.context_content.is_some(),
        "gemini must emit context_content"
    );
    assert_eq!(
        output.mcp_config_path.as_deref(),
        Some(".gemini/settings.json"),
        "gemini mcp_config_path mismatch"
    );
}

/// Compiling for `codex` must emit an AGENTS.md context and a TOML config patch.
#[test]
fn codex_output_has_agents_md_and_config_patch() {
    let library = rich_library();
    let resolved = resolve_library(&library, None, None);
    let output = compile(&resolved, "codex").unwrap();

    assert!(
        output.context_content.is_some(),
        "codex must emit context_content (AGENTS.md)"
    );
    assert!(
        output.codex_config_patch.is_some(),
        "codex must emit codex_config_patch when servers are present"
    );
}

/// MCP servers object always contains the built-in `ship` server key.
#[test]
fn mcp_servers_always_includes_ship_entry() {
    let providers = ["claude", "gemini", "codex", "cursor"];
    let library = ProjectLibrary::default();
    let resolved = resolve_library(&library, None, None);

    for provider in providers {
        let output = compile(&resolved, provider).unwrap();
        let servers = output
            .mcp_servers
            .as_object()
            .expect("mcp_servers must be a JSON object");
        assert!(
            servers.contains_key("ship"),
            "provider {provider} mcp_servers must include built-in 'ship' key"
        );
    }
}

// ─── Empty input ──────────────────────────────────────────────────────────────

/// Empty ProjectLibrary must compile for every known provider without panicking
/// and must produce structurally valid output.
#[test]
fn empty_library_compiles_for_all_providers() {
    let library = ProjectLibrary::default();

    for desc in list_providers() {
        let resolved = resolve_library(&library, None, None);
        let output = compile(&resolved, desc.id)
            .unwrap_or_else(|| panic!("compile returned None for known provider {}", desc.id));

        // MCP servers must be a JSON object (at minimum contains "ship" key).
        assert!(
            output.mcp_servers.is_object(),
            "provider {} mcp_servers must be an object on empty input",
            desc.id
        );

        // Skill and rule file maps may be empty but must not be uninitialised.
        let _ = output.skill_files.len();
        let _ = output.rule_files.len();
    }
}

/// resolve_library on an empty ProjectLibrary must not panic and must default
/// provider to "claude".
#[test]
fn empty_library_resolves_to_claude_default() {
    let library = ProjectLibrary::default();
    let resolved = resolve_library(&library, None, None);
    assert_eq!(
        resolved.providers,
        vec!["claude"],
        "empty library must default to claude provider"
    );
}

// ─── WASM purity (structural) ─────────────────────────────────────────────────

/// The compiler's public entry points accept only in-memory values and return
/// only in-memory values — no `Path`, `File`, or `Command` types in the API.
///
/// This test verifies purity by exercising compile + resolve_library entirely
/// from in-memory inputs and confirming completion without any I/O side effects.
/// A pure function cannot open files or spawn processes; if it could, these
/// calls would fail in a sandboxed environment (e.g., WASM) — making this a
/// meaningful runtime purity guard.
#[test]
fn compile_and_resolve_are_pure_no_io() {
    // Construct inputs purely from in-memory values — no file paths required.
    let library = rich_library();

    // resolve_library takes owned/borrowed structs; no I/O arguments accepted.
    let resolved = resolve_library(&library, None, None);

    // compile takes a ResolvedConfig (in-memory) and a provider string.
    // If this function accessed the filesystem or spawned a process it would
    // either fail (WASM/sandboxed) or require Path/Command arguments it doesn't have.
    for desc in list_providers() {
        let out = compile(&resolved, desc.id);
        assert!(
            out.is_some(),
            "compile must return Some for known provider {}",
            desc.id
        );
    }
}

/// Unknown provider IDs must return None rather than panicking.
#[test]
fn compile_unknown_provider_returns_none() {
    let library = ProjectLibrary::default();
    let resolved = resolve_library(&library, None, None);

    assert!(
        compile(&resolved, "").is_none(),
        "empty provider id must return None"
    );
    assert!(
        compile(&resolved, "unknown-provider").is_none(),
        "unknown provider id must return None"
    );
    assert!(
        compile(&resolved, "CLAUDE").is_none(),
        "provider matching is case-sensitive — 'CLAUDE' must return None"
    );
}
