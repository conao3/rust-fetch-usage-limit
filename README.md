# rust-fetch-usage-limit

A CLI tool that fetches Claude OAuth usage limits and prints JSON output.

## Requirements

- Nix available in your environment
- `ANTHROPIC_OAUTH_API_KEY` set
- Optional `ANTHROPIC_BASE_URL` override when needed

## Verify in devShell

```bash
nix develop -c cargo fmt -- --check
nix develop -c cargo check
```

Run checks

```bash
# Validate missing API key behavior (exit 2)
nix develop -c cargo run --quiet -- claude

# Run with API key
ANTHROPIC_OAUTH_API_KEY=your_token_here \
  nix develop -c cargo run --quiet -- claude
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
