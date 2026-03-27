//! Template variable resolution for skill markdown content.
//!
//! Resolves Jinja2-style template syntax at compile time using pre-loaded
//! variable values. Pure — no I/O, no filesystem access.
//!
//! ## Syntax (standard Jinja2 via MiniJinja)
//!
//! | Construct | Meaning |
//! |-----------|---------|
//! | `{{ var }}` | Scalar substitution |
//! | `{{ obj.field }}` | Dot-path into object |
//! | `{% if var %}…{% endif %}` | Truthy conditional |
//! | `{% if var == "val" %}…{% else %}…{% endif %}` | Equality + else |
//! | `{% for x in arr %}…{% endfor %}` | Iterate array |
//!
//! Undefined variables render as empty string. Template syntax errors
//! fall back to the original content with a warning to stderr.

use minijinja::{Environment, UndefinedBehavior};
use serde_json::Value;
use std::collections::HashMap;

/// Resolve template variables in skill markdown content.
///
/// Returns the rendered string. Undefined variables render as empty.
/// Template syntax errors fall back to the original content.
pub fn resolve_template(content: &str, vars: &HashMap<String, Value>) -> String {
    if !content.contains("{{") && !content.contains("{%") {
        return content.to_string();
    }

    let mut env = Environment::new();
    // Chainable: undefined vars and attribute access on undefined return empty.
    // Skills should degrade gracefully when state hasn't been configured yet.
    env.set_undefined_behavior(UndefinedBehavior::Chainable);
    // No source/loader set — disables {% include %} and {% extends %}.

    match env.render_str(content, vars) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("warning: template error: {e}");
            content.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, val)| (k.to_string(), val.clone()))
            .collect()
    }

    #[test]
    fn no_markers_passthrough() {
        let s = "# Hello\n\nNo vars here.";
        assert_eq!(resolve_template(s, &HashMap::new()), s);
    }

    #[test]
    fn scalar_substitution() {
        let vars = v(&[("name", json!("Alice"))]);
        assert_eq!(resolve_template("Hello {{ name }}!", &vars), "Hello Alice!");
    }

    #[test]
    fn dot_path_substitution() {
        let vars = v(&[("user", json!({"name": "Dr. Mark"}))]);
        assert_eq!(resolve_template("Hi {{ user.name }}", &vars), "Hi Dr. Mark");
    }

    #[test]
    fn deep_dot_path() {
        let vars = v(&[("a", json!({"b": {"c": "deep"}}))]);
        assert_eq!(resolve_template("{{ a.b.c }}", &vars), "deep");
    }

    #[test]
    fn bool_conditional_true() {
        let vars = v(&[("flag", json!(true))]);
        assert_eq!(
            resolve_template("{% if flag %}yes{% endif %}", &vars),
            "yes"
        );
    }

    #[test]
    fn bool_conditional_false() {
        let vars = v(&[("flag", json!(false))]);
        assert_eq!(resolve_template("{% if flag %}yes{% endif %}", &vars), "");
    }

    #[test]
    fn else_branch() {
        let vars = v(&[("flag", json!(false))]);
        assert_eq!(
            resolve_template("{% if flag %}yes{% else %}no{% endif %}", &vars),
            "no"
        );
    }

    #[test]
    fn equality_conditional_match() {
        let vars = v(&[("style", json!("gitmoji"))]);
        assert_eq!(
            resolve_template(r#"{% if style == "gitmoji" %}✨{% endif %}"#, &vars),
            "✨"
        );
    }

    #[test]
    fn equality_conditional_no_match() {
        let vars = v(&[("style", json!("conventional"))]);
        assert_eq!(
            resolve_template(r#"{% if style == "gitmoji" %}✨{% endif %}"#, &vars),
            ""
        );
    }

    #[test]
    fn for_loop_scalars() {
        let vars = v(&[("items", json!(["a", "b", "c"]))]);
        assert_eq!(
            resolve_template("{% for item in items %}{{ item }} {% endfor %}", &vars),
            "a b c "
        );
    }

    #[test]
    fn for_loop_objects() {
        let vars = v(&[("list", json!([{"name": "Alice"}, {"name": "Bob"}]))]);
        assert_eq!(
            resolve_template(
                "{% for item in list %}- {{ item.name }}\n{% endfor %}",
                &vars
            ),
            "- Alice\n- Bob\n"
        );
    }

    #[test]
    fn conditional_inside_loop() {
        let vars = v(&[(
            "list",
            json!([{"name": "Alice", "lead": true}, {"name": "Bob", "lead": false}]),
        )]);
        let tmpl = "{% for item in list %}{{ item.name }}{% if item.lead %} (lead){% endif %}\n{% endfor %}";
        assert_eq!(resolve_template(tmpl, &vars), "Alice (lead)\nBob\n");
    }

    #[test]
    fn missing_var_renders_empty() {
        let out = resolve_template("Hello {{ name }}!", &HashMap::new());
        assert_eq!(out, "Hello !");
    }

    #[test]
    fn bool_renders_as_string() {
        let vars = v(&[("flag", json!(true))]);
        assert_eq!(resolve_template("val: {{ flag }}", &vars), "val: true");
    }

    #[test]
    fn number_renders() {
        let vars = v(&[("count", json!(42))]);
        assert_eq!(resolve_template("count: {{ count }}", &vars), "count: 42");
    }

    #[test]
    fn syntax_error_returns_original() {
        let bad = "{% if %}broken{% endif %}";
        let out = resolve_template(bad, &HashMap::new());
        assert_eq!(out, bad);
    }
}
