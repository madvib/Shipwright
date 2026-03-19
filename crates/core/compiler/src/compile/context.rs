use crate::resolve::ResolvedConfig;

use super::provider::{ContextFile, ProviderDescriptor};

pub(super) fn build_context_content(
    desc: &ProviderDescriptor,
    resolved: &ResolvedConfig,
) -> Option<String> {
    if desc.context_file == ContextFile::None {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();

    // Rules — skip blank content so all-empty rules don't produce a context file
    for rule in &resolved.rules {
        let trimmed = rule.content.trim().to_string();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    // Mode notice
    if let Some(mode) = &resolved.active_agent {
        parts.push(format!("<!-- ship: active mode = {} -->", mode));
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join("\n\n"))
}
