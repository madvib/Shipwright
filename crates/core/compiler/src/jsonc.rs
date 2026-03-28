//! JSONC (JSON with Comments) support.
//!
//! Strips `//` line comments and `/* */` block comments plus trailing commas
//! from a JSONC string, then deserialises with `serde_json`.

/// Strip JSONC comments and trailing commas, returning valid JSON.
///
/// Handles:
/// - `// line comments`
/// - `/* block comments */`
/// - Trailing commas before `]` or `}`
/// - Strings (comments inside strings are preserved)
pub fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();

    while let Some(&(_, ch)) = chars.peek() {
        match ch {
            // String literal — copy verbatim, handling escapes
            '"' => {
                out.push('"');
                chars.next();
                while let Some(&(_, c)) = chars.peek() {
                    if c == '\\' {
                        chars.next();
                        out.push(c);
                        if let Some((_, esc)) = chars.next() {
                            out.push(esc);
                        }
                    } else if c == '"' {
                        out.push('"');
                        chars.next();
                        break;
                    } else {
                        out.push(c);
                        chars.next();
                    }
                }
            }
            // Possible comment start
            '/' => {
                chars.next();
                match chars.peek().map(|&(_, c)| c) {
                    Some('/') => {
                        // Line comment — skip to end of line
                        chars.next();
                        while let Some(&(_, c)) = chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            chars.next();
                        }
                    }
                    Some('*') => {
                        // Block comment — skip to */
                        chars.next();
                        let mut prev = '\0';
                        while let Some((_, c)) = chars.next() {
                            if prev == '*' && c == '/' {
                                break;
                            }
                            prev = c;
                        }
                    }
                    _ => {
                        out.push('/');
                    }
                }
            }
            _ => {
                out.push(ch);
                chars.next();
            }
        }
    }

    // Strip trailing commas before ] or }
    strip_trailing_commas(&out)
}

/// Remove trailing commas: `,` followed by optional whitespace then `]` or `}`.
fn strip_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == ',' {
            // Look ahead past whitespace for ] or }
            let mut j = i + 1;
            while j < len
                && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r')
            {
                j += 1;
            }
            if j < len && (chars[j] == ']' || chars[j] == '}') {
                // Skip the comma, keep the whitespace
                i += 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }

    out
}

/// Parse a JSONC string into a deserializable type.
pub fn from_jsonc_str<T: serde::de::DeserializeOwned>(s: &str) -> serde_json::Result<T> {
    let json = strip_jsonc_comments(s);
    serde_json::from_str(&json)
}

/// Detect config format from file extension.
pub fn is_jsonc_path(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("jsonc") | Some("json")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_line_comments() {
        let input = r#"{
  // This is a comment
  "key": "value" // inline comment
}"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["key"], "value");
    }

    #[test]
    fn strip_block_comments() {
        let input = r#"{
  /* block comment */
  "key": /* inline */ "value"
}"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["key"], "value");
    }

    #[test]
    fn strip_trailing_comma() {
        let input = r#"{
  "a": 1,
  "b": 2,
}"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 2);
    }

    #[test]
    fn strip_trailing_comma_in_array() {
        let input = r#"{ "arr": [1, 2, 3,] }"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["arr"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn preserve_strings_with_slashes() {
        let input = r#"{ "url": "https://example.com/path" }"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["url"], "https://example.com/path");
    }

    #[test]
    fn preserve_escaped_quotes_in_strings() {
        let input = r#"{ "msg": "say \"hello\"" }"#;
        let json = strip_jsonc_comments(input);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["msg"], r#"say "hello""#);
    }

    #[test]
    fn from_jsonc_str_works() {
        let input = r#"{
  // Ship manifest
  "module": {
    "name": "github.com/owner/repo",
    "version": "1.0.0",
  },
}"#;
        let v: serde_json::Value = from_jsonc_str(input).unwrap();
        assert_eq!(v["module"]["name"], "github.com/owner/repo");
    }
}
