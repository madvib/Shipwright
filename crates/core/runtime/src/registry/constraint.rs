use semver::VersionReq;

/// A parsed version constraint from ship.toml [dependencies].
#[derive(Debug, Clone)]
pub enum VersionConstraint {
    Semver(VersionReq),
    Branch(String),
    Commit(String), // 40-char hex
}

/// Parse a version constraint string from ship.toml.
///
/// - 40-char hex → `Commit`
/// - Starts with `^`, `~`, `=`, `>`, `<` or looks like `MAJOR.MINOR.PATCH` → `Semver`
/// - Anything else → `Branch`
pub fn parse_constraint(s: &str) -> anyhow::Result<VersionConstraint> {
    let s = s.trim();

    // 40-char lowercase hex → Commit
    if s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(VersionConstraint::Commit(s.to_string()));
    }

    // Semver range indicators or starts with a digit (bare version like "1.2.3")
    let looks_like_semver = s.starts_with('^')
        || s.starts_with('~')
        || s.starts_with('=')
        || s.starts_with('>')
        || s.starts_with('<')
        || s.chars().next().is_some_and(|c| c.is_ascii_digit());

    if looks_like_semver {
        let req = VersionReq::parse(s)
            .map_err(|e| anyhow::anyhow!("invalid semver constraint {:?}: {}", s, e))?;
        return Ok(VersionConstraint::Semver(req));
    }

    // Everything else is a branch name
    Ok(VersionConstraint::Branch(s.to_string()))
}

/// Strip a leading `v` from a tag for semver comparison.
/// `"v1.0.0"` → `"1.0.0"`, `"1.0.0"` → `"1.0.0"`.
pub fn normalize_version(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_constraint_commit() {
        let sha = "a".repeat(40);
        let c = parse_constraint(&sha).unwrap();
        assert!(matches!(c, VersionConstraint::Commit(_)));
    }

    #[test]
    fn test_parse_constraint_semver_caret() {
        let c = parse_constraint("^1.0.0").unwrap();
        assert!(matches!(c, VersionConstraint::Semver(_)));
    }

    #[test]
    fn test_parse_constraint_semver_tilde() {
        let c = parse_constraint("~1.2.0").unwrap();
        assert!(matches!(c, VersionConstraint::Semver(_)));
    }

    #[test]
    fn test_parse_constraint_semver_bare() {
        let c = parse_constraint("1.2.3").unwrap();
        assert!(matches!(c, VersionConstraint::Semver(_)));
    }

    #[test]
    fn test_parse_constraint_semver_gte() {
        let c = parse_constraint(">=1.0.0").unwrap();
        assert!(matches!(c, VersionConstraint::Semver(_)));
    }

    #[test]
    fn test_parse_constraint_branch() {
        let c = parse_constraint("main").unwrap();
        assert!(matches!(c, VersionConstraint::Branch(ref b) if b == "main"));
    }

    #[test]
    fn test_parse_constraint_branch_feature() {
        let c = parse_constraint("feat/my-feature").unwrap();
        assert!(matches!(c, VersionConstraint::Branch(_)));
    }

    #[test]
    fn test_parse_constraint_invalid_semver() {
        let result = parse_constraint("^not-a-version");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_version_strips_v() {
        assert_eq!(normalize_version("v1.0.0"), "1.0.0");
        assert_eq!(normalize_version("1.0.0"), "1.0.0");
        assert_eq!(normalize_version("v0.2.1-beta"), "0.2.1-beta");
    }
}
