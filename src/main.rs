use clap::{Parser, Subcommand};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::env;
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

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Claude => run_claude().await,
    }
}
