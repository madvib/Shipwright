use anyhow::Result;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

use crate::config::{AuthConfig, ShipConfig};

/// Open `url` in the system's default browser.
fn open_browser(url: &str) {
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/c", "start", url]).spawn();
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    eprintln!("Cannot open browser automatically. Visit: {}", url);
}

/// Generate a PKCE code_verifier from system time + PID entropy.
///
/// Uses the "plain" challenge method (challenge == verifier) to avoid
/// requiring sha2/base64 dependencies.
/// TODO: switch to S256 once sha2 + base64 are added to Cargo.toml.
fn generate_code_verifier() -> String {
    use std::time::SystemTime;
    let t = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id() as u128;
    let mut val = t ^ pid.wrapping_shl(32) ^ pid.wrapping_shr(7);
    // URL-safe alphabet for PKCE verifiers (RFC 7636 §4.1)
    let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::with_capacity(64);
    for _ in 0..64 {
        result.push(chars[(val % 64) as usize] as char);
        // LCG step
        val = val
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
    }
    result
}

/// Extract the `code` query parameter from an HTTP GET request line.
/// e.g. "GET /callback?code=abc123&state=xyz HTTP/1.1"
fn parse_code_from_request(request: &str) -> Option<String> {
    let line = request.lines().next()?;
    let path = line.split_whitespace().nth(1)?;
    let query = path.splitn(2, '?').nth(1)?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next() == Some("code") {
            return kv.next().map(str::to_string);
        }
    }
    None
}

/// Exchange an authorization code + PKCE verifier for a token via POST.
fn exchange_code_for_token(code: &str, verifier: &str, port: u16) -> Result<String> {
    let body = serde_json::json!({
        "code": code,
        "code_verifier": verifier,
        "redirect_uri": format!("http://127.0.0.1:{}/callback", port),
    });

    let resp: String = ureq::post("https://getship.dev/api/auth/token")
        .header("Content-Type", "application/json")
        .send(body.to_string().as_bytes())
        .map_err(|e| anyhow::anyhow!("Token exchange failed: {}", e))?
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("Failed to read token response: {e}"))?;

    let parsed: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|_| anyhow::anyhow!("Invalid response from auth server"))?;

    if let Some(err) = parsed.get("error").and_then(|v| v.as_str()) {
        anyhow::bail!("Auth failed: {}", err);
    }

    parsed.get("token")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("No token in auth response"))
}

/// `ship login` — PKCE OAuth flow.
///
/// Opens the browser to getship.dev/auth/cli, starts a local callback server,
/// waits up to 60 s for the redirect, exchanges the code for a token, then
/// writes it to `~/.ship/config.toml` with 0600 permissions.
pub fn run_login() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| anyhow::anyhow!("Could not start callback server: {}", e))?;
    let port = listener.local_addr()?.port();

    let verifier = generate_code_verifier();
    // PKCE plain: code_challenge == code_verifier (no hash required)
    let challenge = verifier.clone();
    let state = format!("{:x}", port as u64 ^ 0xdead_beef);

    // TODO: switch to real endpoint once https://getship.dev/auth/cli is live
    let auth_url = format!(
        "https://getship.dev/auth/cli\
        ?code_challenge={challenge}\
        &code_challenge_method=plain\
        &redirect_uri=http://127.0.0.1:{port}/callback\
        &state={state}"
    );

    println!("Opening browser for login...");
    println!("  {}", auth_url);
    open_browser(&auth_url);
    println!("\nWaiting for callback (60 s timeout)...");

    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let code = parse_code_from_request(&request).unwrap_or_default();
            let body =
                "<html><body><h1>Login complete</h1><p>You can close this tab.</p></body></html>";
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = tx.send(code);
        }
    });

    let code = match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(c) if !c.is_empty() => c,
        Ok(_) => anyhow::bail!("No authorization code received. Login aborted."),
        Err(_) => anyhow::bail!("Login timed out (60 s). Run `ship login` to try again."),
    };

    let token = exchange_code_for_token(&code, &verifier, port)?;

    let mut cfg = ShipConfig::load();
    cfg.auth = Some(AuthConfig { token: Some(token) });
    cfg.save()?;

    println!("✓ Logged in. Token stored in ~/.ship/config.toml (0600).");
    Ok(())
}

/// `ship logout` — remove the stored auth token.
pub fn run_logout() -> Result<()> {
    let mut cfg = ShipConfig::load();
    if cfg.auth.as_ref().and_then(|a| a.token.as_ref()).is_none() {
        println!("Not logged in.");
        return Ok(());
    }
    cfg.auth = Some(AuthConfig { token: None });
    cfg.save()?;
    println!("✓ Logged out.");
    Ok(())
}

/// `ship whoami` — print identity and auth token status.
pub fn run_whoami() -> Result<()> {
    let cfg = ShipConfig::load();
    match cfg.identity {
        Some(ref id) if !id.name.is_empty() => {
            println!("{}", id.name);
            if let Some(ref email) = id.email {
                println!("{}", email);
            }
        }
        _ => println!("(no identity — run: ship init --global)"),
    }
    match cfg.auth {
        Some(ref a) if a.token.is_some() => println!("authenticated"),
        _ => println!("not logged in"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code_extracts_code_param() {
        let req = "GET /callback?code=abc123&state=xyz HTTP/1.1\r\nHost: localhost\r\n";
        assert_eq!(parse_code_from_request(req), Some("abc123".to_string()));
    }

    #[test]
    fn parse_code_returns_none_when_absent() {
        let req = "GET /callback?state=xyz HTTP/1.1\r\n";
        assert_eq!(parse_code_from_request(req), None);
    }

    #[test]
    fn generate_code_verifier_length_and_charset() {
        let v = generate_code_verifier();
        assert_eq!(v.len(), 64);
        assert!(
            v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "verifier contains invalid chars: {v}"
        );
    }
}
