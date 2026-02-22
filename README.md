# rust-fetch-usage-limit

A CLI tool that fetches Claude and Codex usage limits and prints JSON output.

## Requirements

- Nix available in your environment
- Claude login credentials available at `~/.claude/.credentials.json` or `ANTHROPIC_OAUTH_API_KEY` set
- Optional `ANTHROPIC_BASE_URL` override when needed

## Verify in devShell

```bash
nix develop -c cargo fmt -- --check
nix develop -c cargo check
```

## Package usage (recommended)

```bash
# Build package
nix build .#default

# Run Claude usage
# Priority order:
# 1) ANTHROPIC_OAUTH_API_KEY
# 2) ~/.claude/.credentials.json (claudeAiOauth.accessToken)
nix run .#default -- claude

# Run with API key
ANTHROPIC_OAUTH_API_KEY=your_token_here \
  nix run .#default -- claude

# Run Codex usage
# Priority order:
# 1) OPENAI_OAUTH_API_KEY + OPENAI_ACCOUNT_ID (or CHATGPT_ACCOUNT_ID)
# 2) ~/.codex/auth.json from codex login
nix run .#default -- codex
```

## Release-style verification

```bash
./scripts/verify-package.sh
```

If `ANTHROPIC_BASE_URL` is not set, the command uses `https://api.anthropic.com`.

## Output

Successful output contains:

- `ok`
- `usage`
- `summary.five_hour`
- `summary.seven_day`
- `summary.seven_day_sonnet`

Failure output contains `ok: false` and `error`.

Exit codes

- 0 success
- 1 request, HTTP, or JSON parsing error
- 2 `ANTHROPIC_OAUTH_API_KEY` is missing
