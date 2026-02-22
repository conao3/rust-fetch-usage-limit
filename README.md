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
