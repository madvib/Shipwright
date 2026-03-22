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
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        match bytes[i] {
            // String literal — copy verbatim, handling escapes
            b'"' => {
                out.push('"');
                i += 1;
                while i < len {
                    if bytes[i] == b'\\' && i + 1 < len {
                        out.push(bytes[i] as char);
                        out.push(bytes[i + 1] as char);
                        i += 2;
                    } else if bytes[i] == b'"' {
                        out.push('"');
                        i += 1;
                        break;
                    } else {
                        out.push(bytes[i] as char);
                        i += 1;
                    }
                }
            }
            // Possible comment start
            b'/' if i + 1 < len => {
                if bytes[i + 1] == b'/' {
                    // Line comment — skip to end of line
                    i += 2;
                    while i < len && bytes[i] != b'\n' {
                        i += 1;
                    }
                } else if bytes[i + 1] == b'*' {
                    // Block comment — skip to */
                    i += 2;
                    while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < len {
                        i += 2; // skip */
                    }
                } else {
                    out.push(bytes[i] as char);
                    i += 1;
                }
            }
            _ => {
                out.push(bytes[i] as char);
                i += 1;
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
