//! CI test: assert JSON schemas in schemas/ match compiler types.
//!
//! These tests catch drift between the published schemas and the compiler's
//! actual type definitions. If a schema test fails, update the schema to
//! match the compiler (not the other way around).

use serde_json::Value as Json;

fn load_schema(name: &str) -> Json {
    let path = format!("{}/../../../schemas/{}", env!("CARGO_MANIFEST_DIR"), name);
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Schema not found: {}", path));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", name, e))
}

fn schema_enum_values(schema: &Json, path: &[&str]) -> Vec<String> {
    let mut node = schema;
    for &key in path {
        node = &node[key];
    }
    node.as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

fn schema_object_keys(schema: &Json, path: &[&str]) -> Vec<String> {
    let mut node = schema;
    for &key in path {
        node = &node[key];
    }
    node.as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

fn schema_object_keys_from(node: &Json, path: &[&str]) -> Vec<String> {
    let mut current = node;
    for &key in path {
        current = &current[key];
    }
    current
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

fn schema_has_key(schema: &Json, path: &[&str]) -> bool {
    let mut node = schema;
    for &key in path {
        if let Some(next) = node.get(key) {
            node = next;
        } else {
            return false;
        }
    }
    true
}

// ── Provider enum consistency ────────────────────────────────────────────────

#[test]
fn agent_schema_providers_match_compiler() {
    let schema = load_schema("agent.schema.json");
    let schema_providers = schema_enum_values(
        &schema,
        &[
            "properties",
            "agent",
            "properties",
            "providers",
            "items",
            "enum",
        ],
    );

    let compiler_providers: Vec<&str> = crate::compile::list_providers()
        .iter()
        .map(|p| p.id)
        .collect();

    for p in &compiler_providers {
        assert!(
            schema_providers.contains(&p.to_string()),
            "Compiler has provider '{}' but agent.schema.json is missing it",
            p
        );
    }
}

#[test]
fn ship_schema_providers_match_compiler() {
    let schema = load_schema("ship.schema.json");
    let schema_providers = schema_enum_values(
        &schema,
        &[
            "properties",
            "project",
            "properties",
            "providers",
            "items",
            "enum",
        ],
    );

    let compiler_providers: Vec<&str> = crate::compile::list_providers()
        .iter()
        .map(|p| p.id)
        .collect();

    for p in &compiler_providers {
        assert!(
            schema_providers.contains(&p.to_string()),
            "Compiler has provider '{}' but ship.schema.json is missing it",
            p
        );
    }
}

// ── MCP schema fields match McpServerConfig ──────────────────────────────────

#[test]
fn mcp_schema_has_per_server_provider_fields() {
    let schema = load_schema("mcp.schema.json");
    let server_props = schema_object_keys(&schema, &["$defs", "McpServerEntry", "properties"]);

    let required_fields = [
        "codex_enabled_tools",
        "codex_disabled_tools",
        "gemini_trust",
        "gemini_include_tools",
        "gemini_exclude_tools",
        "gemini_timeout_ms",
        "cursor_env_file",
    ];

    for field in &required_fields {
        assert!(
            server_props.contains(&field.to_string()),
            "McpServerConfig has field '{}' but mcp.schema.json is missing it",
            field
        );
    }
}

#[test]
fn mcp_schema_has_conditional_required() {
    let schema = load_schema("mcp.schema.json");
    // stdio (else branch) requires command
    let else_required =
        schema_enum_values(&schema, &["$defs", "McpServerEntry", "else", "required"]);
    assert!(
        else_required.contains(&"command".to_string()),
        "mcp.schema.json: stdio servers must require 'command'"
    );
    // http/sse (then branch) requires url
    let then_required =
        schema_enum_values(&schema, &["$defs", "McpServerEntry", "then", "required"]);
    assert!(
        then_required.contains(&"url".to_string()),
        "mcp.schema.json: http/sse servers must require 'url'"
    );
}

// ── Permissions schema fields match Permissions struct ────────────────────────

#[test]
fn permissions_schema_preset_fields_match_loader() {
    let schema = load_schema("permissions.schema.json");
    let preset_props = schema_object_keys(&schema, &["$defs", "PermissionPreset", "properties"]);

    // These 5 fields are what the permissions system reads
    for field in [
        "default_mode",
        "tools_allow",
        "tools_deny",
        "tools_ask",
        "additional_directories",
    ] {
        assert!(
            preset_props.contains(&field.to_string()),
            "permissions.schema.json missing {}",
            field
        );
    }
}

// ── ship.schema.json ─────────────────────────────────────────────────────────

#[test]
fn ship_schema_has_provider_defaults() {
    let schema = load_schema("ship.schema.json");
    let project_props = schema_object_keys(&schema, &["properties", "project", "properties"]);

    assert!(
        project_props.contains(&"provider_defaults".to_string()),
        "ship.schema.json project section missing provider_defaults"
    );
}

#[test]
fn ship_schema_has_modes() {
    let schema = load_schema("ship.schema.json");
    let project_props = schema_object_keys(&schema, &["properties", "project", "properties"]);
    assert!(
        project_props.contains(&"modes".to_string()),
        "ship.schema.json project section missing modes"
    );

    // ModeConfig $def has required fields matching compiler's ModeConfig struct
    let mode_props = schema_object_keys(&schema, &["$defs", "ModeConfig", "properties"]);
    for field in [
        "id",
        "name",
        "mcp_servers",
        "skills",
        "rules",
        "permissions",
        "target_agents",
    ] {
        assert!(
            mode_props.contains(&field.to_string()),
            "ModeConfig $def missing field '{}'",
            field
        );
    }
}

// ── agent.schema.json ────────────────────────────────────────────────────────

// Hooks are provider-specific — configured via provider_settings, not schema-level.
// Ship will add hook schemas when the runtime supports hooks natively.

// ── Schema version fields ────────────────────────────────────────────────────

#[test]
fn all_schemas_have_version_field() {
    for name in [
        "agent.schema.json",
        "ship.schema.json",
        "mcp.schema.json",
        "permissions.schema.json",
    ] {
        let schema = load_schema(name);
        assert!(
            schema_has_key(&schema, &["properties", "schema_version"]),
            "{} missing schema_version property",
            name
        );
    }
}

// ── Schema URLs in schemas.rs match actual files ─────────────────────────────

#[test]
fn schema_url_constants_cover_all_providers() {
    for provider in crate::compile::list_providers() {
        let found = crate::PROVIDER_SCHEMAS
            .iter()
            .any(|(id, _)| *id == provider.id);
        assert!(
            found,
            "Provider '{}' missing from PROVIDER_SCHEMAS in schemas.rs",
            provider.id
        );
    }
}

// ── ToolPattern $def exists ──────────────────────────────────────────────────

#[test]
fn agent_schema_has_tool_pattern_def() {
    let schema = load_schema("agent.schema.json");
    assert!(
        schema_has_key(&schema, &["$defs", "ToolPattern"]),
        "agent.schema.json missing ToolPattern $def"
    );
}

#[test]
fn permissions_schema_has_tool_pattern_def() {
    let schema = load_schema("permissions.schema.json");
    assert!(
        schema_has_key(&schema, &["$defs", "ToolPattern"]),
        "permissions.schema.json missing ToolPattern $def"
    );
}

// ── provider_settings managed-key exclusion ────────────────────────────────

/// Verify that the `allOf` exclusion in agent.schema.json provider_settings
/// matches `PROVIDER_MANAGED_KEYS` in schemas.rs.
#[test]
fn agent_schema_provider_settings_exclude_managed_keys() {
    let schema = load_schema("agent.schema.json");
    for (provider_id, expected_keys) in crate::PROVIDER_MANAGED_KEYS {
        if expected_keys.is_empty() {
            continue; // Cursor has no exclusion (no upstream schema)
        }
        // Navigate to provider_settings.<provider>.allOf[1].properties
        let allof = &schema["properties"]["provider_settings"]["properties"][provider_id]["allOf"];
        let excluded = schema_object_keys_from(
            allof.get(1).unwrap_or_else(|| {
                panic!(
                    "agent.schema.json: provider_settings.{} missing allOf[1]",
                    provider_id
                )
            }),
            &["properties"],
        );
        for &key in *expected_keys {
            assert!(
                excluded.contains(&key.to_string()),
                "agent.schema.json: provider_settings.{} missing exclusion for Ship-managed key '{}'",
                provider_id,
                key
            );
        }
    }
}

/// Verify that the `allOf` exclusion in ship.schema.json provider_defaults
/// matches `PROVIDER_MANAGED_KEYS` in schemas.rs.
#[test]
fn ship_schema_provider_defaults_exclude_managed_keys() {
    let schema = load_schema("ship.schema.json");
    for (provider_id, expected_keys) in crate::PROVIDER_MANAGED_KEYS {
        if expected_keys.is_empty() {
            continue;
        }
        let allof = &schema["properties"]["project"]["properties"]["provider_defaults"]["properties"]
            [provider_id]["allOf"];
        let excluded = schema_object_keys_from(
            allof.get(1).unwrap_or_else(|| {
                panic!(
                    "ship.schema.json: provider_defaults.{} missing allOf[1]",
                    provider_id
                )
            }),
            &["properties"],
        );
        for &key in *expected_keys {
            assert!(
                excluded.contains(&key.to_string()),
                "ship.schema.json: provider_defaults.{} missing exclusion for Ship-managed key '{}'",
                provider_id,
                key
            );
        }
    }
}
