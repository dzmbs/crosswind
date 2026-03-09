<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="site/public/crosswind-wordmark-dark.svg">
    <source media="(prefers-color-scheme: light)" srcset="site/public/crosswind-wordmark-light.svg">
    <img src="site/public/crosswind-wordmark-light.svg" alt="crosswind" height="52" />
  </picture>
</p>

<p align="center">Google Flights from your terminal. Built for humans and AI agents.</p>

<p align="center">
  <a href="https://github.com/dzmbs/crosswind/releases"><img src="https://img.shields.io/badge/version-v0.1.0-0EA5E9" alt="version" /></a>
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

# Round-trip
crosswind BEG JFK apr1 -r apr16

# Nonstop only, business class
crosswind LAX JFK tomorrow --max-stops 0 -c business

# Compare multiple destinations
crosswind BEG JFK,LHR,CDG apr1

# JSON output (piped = auto JSON, or force with --json)
crosswind BEG JFK apr1 | jq '.data.flights[0].price'

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
| `--ret` | `-r` | Return date | — |
| `--cabin` | `-c` | economy, premium, business, first | economy |
| `--adults` | `-p` | Passengers (1-9) | 1 |
| `--max-stops` | — | Max connections (0 = nonstop) | — |
| `--currency` | — | ISO 4217 code | USD |
| `--lang` | — | Language | en |
| `--timeout` | — | Seconds | 30 |
| `--json` | — | Force JSON output | — |
| `--open` | — | Open in browser | — |

## Output

- **Terminal**: styled table with green highlighting for "Best flights"
- **Piped**: JSON envelope with `{"v":1, "status":"ok", "cmd":"search", "data":{...}, "timing_ms":N}`
- **`--json`**: force JSON in terminal

```bash
# Count flights
crosswind BEG JFK apr1 | jq '.data.flights | length'

# Cheapest price
crosswind BEG JFK apr1 | jq '.data.flights[0].price'

# All nonstop options
crosswind LAX JFK apr1 | jq '[.data.flights[] | select(.stops == 0)]'
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

- **stdout** = data only (JSON or table)
- **stderr** = diagnostics (timing, hints)
- Semantic exit codes, no output parsing needed
- Structured error envelopes with reason codes and hints
- Consistent JSON envelope with version, status, timing

```bash
RESULT=$(crosswind BEG JFK apr1 --json 2>/dev/null)
if [ $? -eq 0 ]; then
  echo "$RESULT" | jq '.data.flights[0]'
fi
```

## How it works

1. Encodes search params as Protocol Buffers (matching Google's internal format)
2. Fetches the Google Flights page with Chrome TLS fingerprinting
3. Extracts structured data from the embedded JSON payload in the HTML
4. Returns parsed flights with prices, times, airlines, segments, and carbon data

## Building

```bash
cargo build --release
# Binary: target/release/crosswind
```

Requires Rust 1.88+. The build compiles a `.proto` schema via `prost-build`.

## Documentation

Full documentation at the [docs site](https://github.com/dzmbs/crosswind) (coming soon):

- [Getting Started](/site/pages/introduction/getting-started.mdx)
- [Date Formats](/site/pages/usage/dates.mdx)
- [Output Modes](/site/pages/reference/output.mdx)
- [Agent Integration](/site/pages/reference/agent-integration.mdx)
- [Limitations](/site/pages/reference/limitations.mdx)

## License

[MIT](LICENSE)
