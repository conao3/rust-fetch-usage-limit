# rust-fetch-usage-limit

Claude の OAuth usage limit を取得して JSON 出力する CLI です。

## 前提

- Nix が使えること
- `ANTHROPIC_OAUTH_API_KEY` が設定されていること
- 必要に応じて `ANTHROPIC_BASE_URL` を設定できること

## devShell での動作確認

```bash
nix develop -c cargo fmt -- --check
nix develop -c cargo check
```

実行確認

```bash
# APIキー未設定時の確認（exit 2）
nix develop -c cargo run --quiet -- claude

# APIキー設定時
ANTHROPIC_OAUTH_API_KEY=your_token_here \
  nix develop -c cargo run --quiet -- claude
```

`ANTHROPIC_BASE_URL` は未設定時に `https://api.anthropic.com` を使います。

## 出力

成功時は次の形式で出力します。

- `ok`
- `usage`
- `summary.five_hour`
- `summary.seven_day`
- `summary.seven_day_sonnet`

失敗時は `ok: false` と `error` を返します。

終了コード

- 0 成功
- 1 通信エラーやHTTPエラー、JSON解析エラー
- 2 `ANTHROPIC_OAUTH_API_KEY` 未設定
