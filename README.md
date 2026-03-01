# Pratrol

Intelligent PR triage for modern maintainers. Pratrol is a GitHub App that automatically analyzes pull requests and posts a triage comment with trust and quality scores.

## How it works

1. A PR is opened on an installed repository
2. Pratrol receives the webhook, fetches the diff, commits, and author profile
3. Scores the author profile (account age, contributions, merge history)
4. Sends the diff to Mistral for code quality analysis
5. Posts a comment with combined trust score and summary

## Quick start

```bash
export GITHUB_APP_ID=123456
export GITHUB_PRIVATE_KEY_PATH=./private-key.pem
export GITHUB_WEBHOOK_SECRET=your_secret
export MISTRAL_API_KEY=your_key
export RUST_LOG=debug
export MISTRAL_MODEL=mistral-small-latest

cargo run
```

## Deploy

Production configs are in `deploy/`:

- `pratrol.service` — systemd unit
- `nginx-api.pratrol.com.conf` — nginx reverse proxy
- `pratrol.env.example` — environment variables template

## License

MPL-2.0
