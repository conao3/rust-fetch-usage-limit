# rust-fetch-usage-limit

A CLI tool that fetches usage-limit information for Claude and Codex and prints JSON output.

## Features

- Fetch Claude OAuth usage with the `claude` subcommand
- Fetch Codex usage with the `codex` subcommand
- Print structured JSON with `ok`, `usage`, and `summary`

## Requirements

- Nix available in your environment
- Use `nix develop` for local Rust development tasks

## Quick start

```bash
# Show CLI help
nix run .#default -- --help

# Fetch Claude usage
nix run .#default -- claude

# Fetch Codex usage
nix run .#default -- codex
```

## Authentication resolution order

### Claude

1. `ANTHROPIC_OAUTH_API_KEY`
2. `~/.claude/.credentials.json` field `claudeAiOauth.accessToken`

Optional settings

- `ANTHROPIC_BASE_URL`
- Default is `https://api.anthropic.com`

Example

```bash
ANTHROPIC_OAUTH_API_KEY=your_token \
  nix run .#default -- claude
```

### Codex

1. `OPENAI_OAUTH_API_KEY` plus `OPENAI_ACCOUNT_ID` or `CHATGPT_ACCOUNT_ID`
2. `~/.codex/auth.json` fields `tokens.access_token` and `tokens.account_id`

Optional settings

- `CHATGPT_BASE_URL`
- Default is `https://chatgpt.com`

Example

```bash
OPENAI_OAUTH_API_KEY=your_token \
OPENAI_ACCOUNT_ID=your_account_id \
  nix run .#default -- codex
```

## Output

On success

- `ok: true`
- `usage`: raw API response
- `summary`: normalized window summary

On failure

- `ok: false`
- `error`
- `response_body` for non-success HTTP responses

### Example output (redacted)

```json
{
  "ok": true,
  "summary": {
    "five_hour": {
      "percent_left": 96.0,
      "resets_at": "<redacted-timestamp>"
    },
    "seven_day": {
      "percent_left": 100.0,
      "resets_at": "<redacted-timestamp>"
    },
    "seven_day_sonnet": {
      "percent_left": 100.0,
      "resets_at": "<redacted-timestamp>"
    }
  },
  "usage": {
    "extra_usage": {
      "is_enabled": false,
      "monthly_limit": null,
      "used_credits": null,
      "utilization": null
    },
    "five_hour": {
      "resets_at": "<redacted-timestamp>",
      "utilization": 4.0
    },
    "iguana_necktie": null,
    "seven_day": {
      "resets_at": "<redacted-timestamp>",
      "utilization": 0.0
    },
    "seven_day_cowork": null,
    "seven_day_oauth_apps": null,
    "seven_day_opus": null,
    "seven_day_sonnet": {
      "resets_at": "<redacted-timestamp>",
      "utilization": 0.0
    }
  }
}
```

### Example output for `codex` (redacted)

```json
{
  "ok": true,
  "summary": {
    "five_hour": {
      "limit_window_seconds": 18000,
      "reset_after_seconds": 4266,
      "reset_at": "<redacted-unix-timestamp>",
      "used_percent": 96
    },
    "seven_day": {
      "limit_window_seconds": 604800,
      "reset_after_seconds": 517980,
      "reset_at": "<redacted-unix-timestamp>",
      "used_percent": 84
    }
  },
  "usage": {
    "account_id": "<redacted-account-id>",
    "additional_rate_limits": null,
    "code_review_rate_limit": {
      "allowed": true,
      "limit_reached": false,
      "primary_window": {
        "limit_window_seconds": 604800,
        "reset_after_seconds": 604800,
        "reset_at": "<redacted-unix-timestamp>",
        "used_percent": 0
      },
      "secondary_window": null
    },
    "credits": {
      "approx_cloud_messages": [0, 0],
      "approx_local_messages": [0, 0],
      "balance": "0",
      "has_credits": false,
      "unlimited": false
    },
    "email": "<redacted-email>",
    "plan_type": "plus",
    "promo": null,
    "rate_limit": {
      "allowed": true,
      "limit_reached": false,
      "primary_window": {
        "limit_window_seconds": 18000,
        "reset_after_seconds": 4266,
        "reset_at": "<redacted-unix-timestamp>",
        "used_percent": 96
      },
      "secondary_window": {
        "limit_window_seconds": 604800,
        "reset_after_seconds": 517980,
        "reset_at": "<redacted-unix-timestamp>",
        "used_percent": 84
      }
    },
    "user_id": "<redacted-user-id>"
  }
}
```

Summary fields

For `claude`

- `five_hour`
- `seven_day`
- `seven_day_sonnet`

For `codex`

- `five_hour` from `rate_limit.primary_window`
- `seven_day` from `rate_limit.secondary_window`

## Exit codes

- `0`: success
- `1`: request failure, HTTP error, JSON parse error, or runtime error
- `2`: missing or invalid auth configuration

## Development checks

```bash
nix develop -c cargo fmt -- --check
nix develop -c cargo check
```

## Package verification

```bash
./scripts/verify-package.sh
```
