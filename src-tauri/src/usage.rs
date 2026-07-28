use serde::{Deserialize, Serialize};

use crate::error::{AppError, ErrorKind, Result};
use crate::profiles::Profile;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20";
/// Don't use a token that expires within the next minute.
const EXPIRY_BUFFER_MS: u64 = 60_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowUsage {
    /// Percent, 0–100.
    pub utilization: f64,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UsageStatus {
    Ok,
    /// Profile has never logged into Claude Code — no token to query with.
    NoCredentials,
    /// Stored access token is expired; we deliberately never refresh it
    /// ourselves (rotating the refresh token could log out the user's CLI).
    TokenExpired,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub status: UsageStatus,
    pub five_hour: Option<WindowUsage>,
    pub seven_day: Option<WindowUsage>,
    pub message: Option<String>,
}

impl Usage {
    fn empty(status: UsageStatus, message: Option<String>) -> Self {
        Self {
            status,
            five_hour: None,
            seven_day: None,
            message,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OauthBlob>,
}

#[derive(Debug, Deserialize)]
struct OauthBlob {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<u64>,
}

enum Token {
    Missing,
    Expired,
    Valid(String),
}

/// Read the profile's Claude Code OAuth access token, read-only.
/// Never log or embed the token in errors.
fn read_token(profile: &Profile) -> Result<Token> {
    let raw = read_credentials_blob(profile)?;
    let Some(raw) = raw else {
        return Ok(Token::Missing);
    };
    let parsed: CredentialsFile = serde_json::from_str(&raw)
        .map_err(|_| AppError::new(ErrorKind::Internal, "unrecognized credentials format"))?;
    let Some(oauth) = parsed.claude_ai_oauth else {
        return Ok(Token::Missing);
    };
    let Some(token) = oauth.access_token.filter(|t| !t.is_empty()) else {
        return Ok(Token::Missing);
    };
    if let Some(expires_at) = oauth.expires_at {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        if now_ms + EXPIRY_BUFFER_MS >= expires_at {
            return Ok(Token::Expired);
        }
    }
    Ok(Token::Valid(token))
}

#[cfg(target_os = "macos")]
fn read_credentials_blob(profile: &Profile) -> Result<Option<String>> {
    let service = if profile.is_default {
        "Claude Code-credentials".to_string()
    } else {
        format!(
            "Claude Code-credentials-{}",
            crate::profiles::keychain_suffix(&profile.code.config_dir)
        )
    };
    let out = std::process::Command::new("security")
        .args(["find-generic-password", "-s", &service, "-w"])
        .output()?;
    if !out.status.success() {
        return Ok(None); // item not found → never logged in
    }
    Ok(Some(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
fn read_credentials_blob(profile: &Profile) -> Result<Option<String>> {
    match std::fs::read_to_string(profile.code.config_dir.join(".credentials.json")) {
        Ok(raw) => Ok(Some(raw)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, Deserialize)]
struct ApiWindow {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiUsageResponse {
    five_hour: Option<ApiWindow>,
    seven_day: Option<ApiWindow>,
}

fn to_window(w: Option<ApiWindow>) -> Option<WindowUsage> {
    let w = w?;
    Some(WindowUsage {
        utilization: w.utilization?,
        resets_at: w.resets_at,
    })
}

pub async fn get_usage(profile: &Profile) -> Result<Usage> {
    let token = match read_token(profile)? {
        Token::Missing => return Ok(Usage::empty(UsageStatus::NoCredentials, None)),
        Token::Expired => {
            return Ok(Usage::empty(
                UsageStatus::TokenExpired,
                Some("login token expired — run claude once to refresh it".into()),
            ))
        }
        Token::Valid(t) => t,
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::new(ErrorKind::Internal, e.to_string()))?;
    let response = client
        .get(USAGE_URL)
        .bearer_auth(&token)
        .header("anthropic-beta", OAUTH_BETA_HEADER)
        .send()
        .await
        .map_err(|e| AppError::new(ErrorKind::Io, format!("usage request failed: {e}")))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Ok(Usage::empty(
            UsageStatus::TokenExpired,
            Some("login token rejected — run claude once to refresh it".into()),
        ));
    }
    if !response.status().is_success() {
        return Ok(Usage::empty(
            UsageStatus::Error,
            Some(format!("usage API returned {}", response.status())),
        ));
    }
    let parsed: ApiUsageResponse = response
        .json()
        .await
        .map_err(|e| AppError::new(ErrorKind::Internal, format!("usage parse error: {e}")))?;

    Ok(Usage {
        status: UsageStatus::Ok,
        five_hour: to_window(parsed.five_hour),
        seven_day: to_window(parsed.seven_day),
        message: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_usage_response_fixture() {
        // Shape observed via claude-swap's parser of the same endpoint.
        let raw = r#"{
            "five_hour": {"utilization": 32.5, "resets_at": "2026-07-28T18:00:00+00:00"},
            "seven_day": {"utilization": 61.0, "resets_at": "2026-08-01T09:00:00+00:00"},
            "extra_usage": {"used_credits": null, "monthly_limit": null, "utilization": null}
        }"#;
        let parsed: ApiUsageResponse = serde_json::from_str(raw).unwrap();
        let five = to_window(parsed.five_hour).unwrap();
        assert_eq!(five.utilization, 32.5);
        assert!(five.resets_at.unwrap().starts_with("2026-07-28"));
        assert_eq!(to_window(parsed.seven_day).unwrap().utilization, 61.0);
    }

    #[test]
    fn missing_windows_become_none() {
        let parsed: ApiUsageResponse = serde_json::from_str("{}").unwrap();
        assert!(to_window(parsed.five_hour).is_none());
        assert!(to_window(parsed.seven_day).is_none());
    }

    #[test]
    fn credentials_blob_parses() {
        let raw =
            r#"{"claudeAiOauth": {"accessToken": "tok", "refreshToken": "r", "expiresAt": 1}}"#;
        let parsed: CredentialsFile = serde_json::from_str(raw).unwrap();
        let oauth = parsed.claude_ai_oauth.unwrap();
        assert_eq!(oauth.access_token.as_deref(), Some("tok"));
        assert_eq!(oauth.expires_at, Some(1));
    }
}
