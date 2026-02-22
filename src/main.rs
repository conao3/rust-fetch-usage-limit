use clap::{Parser, Subcommand};
use regex::Regex;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use uuid::Uuid;

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
    /// Fetch Codex usage status through openclaw session_status
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

fn left(v: Option<f64>) -> Option<f64> {
    v.map(|n| (100.0 - n).max(0.0))
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(_) => println!("{{\"ok\":false,\"error\":\"failed to serialize output\"}}"),
    }
}

async fn run_claude() -> ExitCode {
    let base_url =
        env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".to_string());

    let api_key = match env::var("ANTHROPIC_OAUTH_API_KEY") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            print_json(&json!({"ok": false, "error": "ANTHROPIC_OAUTH_API_KEY is not set"}));
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

fn parse_status_text(text: &str) -> Value {
    let mut out = Map::new();

    let model_re = Regex::new(r"Model:\s*([\w./-]+)").ok();
    let tokens_re = Regex::new(r"Tokens:\s*([^\s]+)\s+in\s*/\s*([^\s]+)\s+out").ok();
    let context_re = Regex::new(r"Context:\s*([^\s]+)/([^\s]+)\s*\((\d+)%\)").ok();
    let usage_re =
        Regex::new(r"Usage:\s*5h\s+(\d+)%\s+left\s+⏱([^·]+?)\s+·\s+Day\s+(\d+)%\s+left\s+⏱([^\n]+)")
            .ok();
    let session_re = Regex::new(r"Session:\s*([^\s•]+)").ok();

    if let Some(re) = model_re
        && let Some(c) = re.captures(text)
    {
        out.insert("model".to_string(), json!(c[1].to_string()));
    }
    if let Some(re) = tokens_re
        && let Some(c) = re.captures(text)
    {
        out.insert(
            "tokens".to_string(),
            json!({"input": c[1].to_string(), "output": c[2].to_string()}),
        );
    }
    if let Some(re) = context_re
        && let Some(c) = re.captures(text)
    {
        let percent = c[3].parse::<i64>().unwrap_or(0);
        out.insert(
            "context".to_string(),
            json!({"used": c[1].to_string(), "total": c[2].to_string(), "percent": percent}),
        );
    }
    if let Some(re) = usage_re
        && let Some(c) = re.captures(text)
    {
        let p5 = c[1].parse::<i64>().unwrap_or(0);
        let pday = c[3].parse::<i64>().unwrap_or(0);
        out.insert(
            "ratelimit".to_string(),
            json!({
                "5h_percent_left": p5,
                "5h_reset_in": c[2].trim().to_string(),
                "daily_percent_left": pday,
                "daily_reset_in": c[4].trim().to_string()
            }),
        );
    }
    if let Some(re) = session_re
        && let Some(c) = re.captures(text)
    {
        out.insert("session".to_string(), json!(c[1].to_string()));
    }

    Value::Object(out)
}

fn get_or_create_session_id() -> Result<String, String> {
    let session_id_path = env::var("CODEX_USAGE_SESSION_ID_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home)
                .join(".openclaw")
                .join("workspace")
                .join(".pi")
                .join("system-codex-usage-limit.uuid")
        });

    if let Some(parent) = session_id_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create session id directory: {e}"))?;
    }

    if session_id_path.exists() {
        let existing = fs::read_to_string(&session_id_path)
            .map_err(|e| format!("failed to read session id file: {e}"))?;
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let sid = Uuid::new_v4().to_string();
    fs::write(&session_id_path, format!("{sid}\n"))
        .map_err(|e| format!("failed to write session id file: {e}"))?;
    Ok(sid)
}

fn run_codex() -> ExitCode {
    let session_name = "system-codex-usage-limit";
    let prompt =
        "session_statusを実行して、その生の出力テキストのみを返してください。追加のコメントは不要です。";
    let agent_id = env::var("CODEX_USAGE_AGENT_ID").unwrap_or_else(|_| "codex-usage".to_string());

    let session_id = match get_or_create_session_id() {
        Ok(sid) => sid,
        Err(e) => {
            print_json(&json!({"ok": false, "error": e}));
            return ExitCode::from(1);
        }
    };

    let output = match Command::new("openclaw")
        .arg("agent")
        .arg("--agent")
        .arg(&agent_id)
        .arg("--session-id")
        .arg(&session_id)
        .arg("--message")
        .arg(prompt)
        .arg("--json")
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::NotFound {
                "openclaw not found".to_string()
            } else {
                format!("command failed: {e}")
            };
            print_json(&json!({"ok": false, "error": msg}));
            return ExitCode::from(2);
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        print_json(&json!({
            "ok": false,
            "error": if stderr.is_empty() { "command failed" } else { &stderr },
        }));
        return ExitCode::from(1);
    }

    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    let data: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            print_json(&json!({
                "ok": false,
                "error": e.to_string(),
                "raw": raw.chars().take(500).collect::<String>()
            }));
            return ExitCode::from(1);
        }
    };

    let status_text = data
        .get("result")
        .and_then(|r| r.get("payloads"))
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let mut out = Map::new();
    out.insert(
        "ok".to_string(),
        json!(data.get("status").and_then(|s| s.as_str()) == Some("ok")),
    );
    out.insert("session_name".to_string(), json!(session_name));
    out.insert("agent_id".to_string(), json!(agent_id));
    out.insert("session_id".to_string(), json!(session_id));
    out.insert("status_text".to_string(), json!(status_text.clone()));

    if let Some(parsed) = parse_status_text(&status_text).as_object() {
        for (k, v) in parsed {
            out.insert(k.clone(), v.clone());
        }
    }

    let ok = out.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    print_json(&Value::Object(out));
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Claude => run_claude().await,
        Commands::Codex => run_codex(),
    }
}
