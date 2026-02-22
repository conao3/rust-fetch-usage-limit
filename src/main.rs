use clap::{Parser, Subcommand};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "fetch-usage-limit")]
#[command(about = "Usage limit utilities", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch Claude OAuth usage limits and print JSON output
    Claude,
    /// Fetch Codex usage limits and print JSON output
    Codex,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UsageWindow {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OAuthUsageResponse {
    five_hour: Option<UsageWindow>,
    seven_day: Option<UsageWindow>,
    seven_day_sonnet: Option<UsageWindow>,
}

#[derive(Debug, Deserialize)]
struct CodexAuthFile {
    tokens: Option<CodexTokens>,
}

#[derive(Debug, Deserialize)]
struct CodexTokens {
    access_token: Option<String>,
    account_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeCredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<ClaudeAiOauth>,
}

#[derive(Debug, Deserialize)]
struct ClaudeAiOauth {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
}

fn left(v: Option<f64>) -> Option<f64> {
    v.map(|n| (100.0 - n).max(0.0))
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(_) => println!("{{\"ok\":false,\"error\":\"failed to serialize output\"}}"),
    }
}

fn read_claude_oauth_token() -> Result<String, String> {
    if let Ok(v) = env::var("ANTHROPIC_OAUTH_API_KEY") {
        let token = v.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    let credentials_path = env::var("CLAUDE_CREDENTIALS_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()))
                .join(".claude/.credentials.json")
        });

    let content = fs::read_to_string(&credentials_path)
        .map_err(|e| format!("failed to read {}: {e}", credentials_path.display()))?;
    let credentials: ClaudeCredentialsFile = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse {}: {e}", credentials_path.display()))?;

    credentials
        .claude_ai_oauth
        .and_then(|o| o.access_token)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            "ANTHROPIC_OAUTH_API_KEY is not set and accessToken was not found in ~/.claude/.credentials.json"
                .to_string()
        })
}

async fn run_claude() -> ExitCode {
    let base_url =
        env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".to_string());

    let api_key = match read_claude_oauth_token() {
        Ok(v) => v,
        Err(e) => {
            print_json(&json!({"ok": false, "error": e}));
            return ExitCode::from(2);
        }
    };

    let url = format!("{}/api/oauth/usage", base_url.trim_end_matches('/'));

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("failed to build HTTP client: {e}")}));
            return ExitCode::from(1);
        }
    };

    let response = match client
        .get(url)
        .header(ACCEPT, "application/json, text/plain, */*")
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "claude-code/2.0.32")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("request failed: {e}")}));
            return ExitCode::from(1);
        }
    };

    let status = response.status();
    let body_text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            print_json(
                &json!({"ok": false, "error": format!("failed to read response body: {e}")}),
            );
            return ExitCode::from(1);
        }
    };

    if !status.is_success() {
        print_json(&json!({
            "ok": false,
            "error": format!("HTTP {}", status.as_u16()),
            "response_body": body_text,
        }));
        return ExitCode::from(1);
    }

    let usage_value: Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("failed to parse JSON: {e}")}));
            return ExitCode::from(1);
        }
    };

    let usage: OAuthUsageResponse =
        serde_json::from_value(usage_value.clone()).unwrap_or(OAuthUsageResponse {
            five_hour: None,
            seven_day: None,
            seven_day_sonnet: None,
        });

    let mut summary: Map<String, Value> = Map::new();
    summary.insert(
        "five_hour".to_string(),
        json!({
            "resets_at": usage.five_hour.as_ref().and_then(|w| w.resets_at.clone()),
            "percent_left": left(usage.five_hour.as_ref().and_then(|w| w.utilization)),
        }),
    );
    summary.insert(
        "seven_day".to_string(),
        json!({
            "resets_at": usage.seven_day.as_ref().and_then(|w| w.resets_at.clone()),
            "percent_left": left(usage.seven_day.as_ref().and_then(|w| w.utilization)),
        }),
    );
    summary.insert(
        "seven_day_sonnet".to_string(),
        json!({
            "resets_at": usage.seven_day_sonnet.as_ref().and_then(|w| w.resets_at.clone()),
            "percent_left": left(usage.seven_day_sonnet.as_ref().and_then(|w| w.utilization)),
        }),
    );

    print_json(&json!({
        "ok": true,
        "usage": usage_value,
        "summary": summary,
    }));

    ExitCode::SUCCESS
}

fn read_codex_auth() -> Result<(String, String), String> {
    if let Ok(access_token) = env::var("OPENAI_OAUTH_API_KEY") {
        let access_token = access_token.trim().to_string();
        if !access_token.is_empty() {
            let account_id = env::var("OPENAI_ACCOUNT_ID")
                .or_else(|_| env::var("CHATGPT_ACCOUNT_ID"))
                .map_err(|_| {
                    "OPENAI_OAUTH_API_KEY is set, but OPENAI_ACCOUNT_ID or CHATGPT_ACCOUNT_ID is missing"
                        .to_string()
                })?;
            let account_id = account_id.trim().to_string();
            if account_id.is_empty() {
                return Err(
                    "OPENAI_OAUTH_API_KEY is set, but OPENAI_ACCOUNT_ID/CHATGPT_ACCOUNT_ID is empty"
                        .to_string(),
                );
            }
            return Ok((access_token, account_id));
        }
    }

    let auth_path = env::var("CODEX_AUTH_FILE").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()))
            .join(".codex/auth.json")
    });

    let content = fs::read_to_string(&auth_path)
        .map_err(|e| format!("failed to read auth file {}: {e}", auth_path.display()))?;
    let auth: CodexAuthFile = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse auth file {}: {e}", auth_path.display()))?;

    let tokens = auth
        .tokens
        .ok_or_else(|| "tokens are missing in codex auth file".to_string())?;
    let access_token = tokens
        .access_token
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "access_token is missing in codex auth file".to_string())?;
    let account_id = tokens
        .account_id
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "account_id is missing in codex auth file".to_string())?;

    Ok((access_token, account_id))
}

async fn run_codex() -> ExitCode {
    let (access_token, account_id) = match read_codex_auth() {
        Ok(v) => v,
        Err(e) => {
            print_json(&json!({"ok": false, "error": e}));
            return ExitCode::from(2);
        }
    };

    let base_url = env::var("CHATGPT_BASE_URL").unwrap_or_else(|_| "https://chatgpt.com".to_string());
    let url = format!("{}/backend-api/wham/usage", base_url.trim_end_matches('/'));

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("failed to build HTTP client: {e}")}));
            return ExitCode::from(1);
        }
    };

    let response = match client
        .get(url)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "codex-cli")
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .header("ChatGPT-Account-Id", account_id)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("request failed: {e}")}));
            return ExitCode::from(1);
        }
    };

    let status = response.status();
    let body_text = match response.text().await {
        Ok(t) => t,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("failed to read response body: {e}")}));
            return ExitCode::from(1);
        }
    };

    if !status.is_success() {
        print_json(&json!({
            "ok": false,
            "error": format!("HTTP {}", status.as_u16()),
            "response_body": body_text,
        }));
        return ExitCode::from(1);
    }

    let usage_value: Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(e) => {
            print_json(&json!({"ok": false, "error": format!("failed to parse JSON: {e}")}));
            return ExitCode::from(1);
        }
    };

    let primary = usage_value
        .get("rate_limit")
        .and_then(|v| v.get("primary_window"))
        .cloned()
        .unwrap_or(Value::Null);
    let secondary = usage_value
        .get("rate_limit")
        .and_then(|v| v.get("secondary_window"))
        .cloned()
        .unwrap_or(Value::Null);

    let out = json!({
        "ok": true,
        "usage": usage_value,
        "summary": {
            "five_hour": primary,
            "seven_day": secondary
        }
    });

    print_json(&out);
    ExitCode::SUCCESS
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Claude => run_claude().await,
        Commands::Codex => run_codex().await,
    }
}
