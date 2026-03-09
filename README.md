<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="site/public/crosswind-wordmark-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="site/public/crosswind-wordmark-light.svg">
    <img src="site/public/crosswind-wordmark-light.svg" alt="crosswind" height="52" />
  </picture>
</p>

<p align="center">Google Flights from your terminal. Built for humans and AI agents.</p>

<p align="center">
  <a href="https://github.com/dzmbs/crosswind/releases"><img src="https://img.shields.io/badge/version-v0.2.0-0EA5E9" alt="version" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="license" /></a>
  <img src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux-lightgrey" alt="platforms" />
</p>

## What is this

A single static binary that searches Google Flights using positional arguments, smart date parsing, and structured JSON output. No API key. No headless browser. No Python.

```bash
crosswind BEG JFK apr1
```

## Install

```bash
# One-command install
curl -fsSL https://raw.githubusercontent.com/dzmbs/crosswind/main/install.sh | bash

# Or with cargo
cargo install --git https://github.com/dzmbs/crosswind --bin crosswind
```

## Quick start

```bash
# One-way search
crosswind BEG JFK apr1

# Round-trip, business class
crosswind BEG JFK apr1 -r apr16 -c business

# Nonstop only
crosswind LAX JFK tomorrow --nonstop

# Compare multiple destinations
crosswind BEG JFK,LHR,CDG apr1

# Shortest flights, top 5
crosswind BEG JFK apr1 --sort duration --top 5

# Under 12 hours only
crosswind BEG JFK apr1 --max-duration 720

# JSON output (auto when piped, or force with --json)
crosswind BEG JFK apr1 | jq '.data.flights[0].price'

# Just the cheapest price
crosswind BEG JFK apr1 --quiet

# Open in browser
crosswind BEG JFK apr1 --open
```

## Date formats

| Format | Example |
|---|---|
| Short month + day | `apr1`, `mar28` |
| Slash | `4/1`, `4/1/2026` |
| ISO 8601 | `2026-04-01` |
| Relative | `+7`, `+30` |
| Named | `today`, `tomorrow` |
| Month only | `apr` |

## Flags

| Flag | Short | Description | Default |
|---|---|---|---|
| `--ret` | `-r` | Return date | |
| `--cabin` | `-c` | economy, premium, business, first | economy |
| `--adults` | `-p` | Passengers (1-9) | 1 |
| `--nonstop` | | Nonstop flights only | |
| `--max-stops` | | Max connections (0 = nonstop) | |
| `--max-duration` | | Max flight duration in minutes | |
| `--sort` | | Sort by: price, duration, stops, departure | price |
| `--top` | | Show only the top N results | |
| `--currency` | | ISO 4217 code | USD |
| `--lang` | | Language | en |
| `--timeout` | | Seconds | 30 |
| `--json` | | Force JSON output | |
| `--output` | | Output format: pretty, json, ndjson, quiet | auto |
| `--quiet` | `-q` | Minimal output (cheapest price only) | |
| `--open` | | Open in browser | |

## Output

- **Terminal (TTY)**: styled table, green highlighting for best flights. Header and summary on stderr.
- **Piped**: JSON envelope automatically. `{"v":1, "status":"ok", "cmd":"search", "data":{...}, "timing_ms":N}`
- **`--json`**: force JSON in terminal
- **`--output ndjson`**: one flight per line, for streaming and scripting
- **`--quiet`**: just the cheapest price

```bash
# Count flights
crosswind BEG JFK apr1 | jq '.data.flights | length'

# Cheapest price
crosswind BEG JFK apr1 -q

# Nonstop under 12 hours, sorted by duration
crosswind BEG JFK apr1 --nonstop --max-duration 720 --sort duration | jq '.data.flights[]'

# Stream flights as NDJSON
crosswind BEG JFK apr1 --output ndjson | while read -r line; do echo "$line" | jq .price; done
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error |
| 2 | Validation (bad input) |
| 3 | Network (timeout, DNS, TLS) |
| 4 | Rate limited / blocked |
| 5 | Parse error (page changed, no results) |

## Agent integration

Crosswind is designed for AI agents and scripts:

- **stdout** = data only (JSON envelope or table rows)
- **stderr** = diagnostics only (table header, timing, hints)
- Semantic exit codes for programmatic error handling
- Structured error envelopes with reason codes, hints, and `retryable` flag
- Consistent JSON envelope: `v`, `status`, `cmd`, `data`, `timing_ms`
- NDJSON mode for streaming into pipelines

```bash
# Agent pattern: structured error handling
RESULT=$(crosswind BEG JFK apr1 --json 2>/dev/null)
EXIT=$?
if [ $EXIT -eq 0 ]; then
  echo "$RESULT" | jq '.data.flights[0]'
elif [ $EXIT -eq 4 ]; then
  echo "Rate limited, retrying..."
fi
```

Error envelope:
```json
{
  "v": 1,
  "status": "error",
  "cmd": "search",
  "code": "rate_limited",
  "message": "rate limited by Google",
  "retryable": true,
  "hint": "wait a few minutes before retrying",
  "timing_ms": 1200
}
```

## How it works

1. Encodes search params as Protocol Buffers (matching Google's internal format)
2. Fetches the Google Flights page with Chrome TLS fingerprinting
3. Extracts structured data from the embedded JSON payload in the HTML
4. Filters, sorts, and returns parsed flights with prices, times, airlines, segments, and carbon data

## Building

```bash
cargo build --release
# Binary: target/release/crosswind
```

Requires Rust 1.88+. The build compiles a `.proto` schema via `prost-build`.

## License

[MIT](LICENSE)
