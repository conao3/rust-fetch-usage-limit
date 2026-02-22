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

## Package usage (recommended)

```bash
# Build package
nix build .#default

# Run package
nix run .#default -- claude

# Run with API key
ANTHROPIC_OAUTH_API_KEY=your_token_here \
  nix run .#default -- claude
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
