# LoadMaxx

A blazing-fast CLI HTTP load testing tool built in Rust. Designed to generate massive concurrent load with minimal resource overhead.

## Features

- **GET and POST support** — test static pages, APIs, login flows, and form submissions
- **HTTP/2 support** — auto-negotiates on HTTPS, or force with `--http2`
- **High-concurrency async I/O** — powered by tokio and reqwest
- **Detailed latency stats** — avg, min, max, P50, P90, P99
- **CSV request logging** — per-request timestamp, status code, latency, and errors
- **Progress bar** — real-time completion tracking
- **Status code breakdown** — color-coded 2xx/3xx/4xx/5xx summary
- **Error aggregation** — grouped error counts for quick diagnosis
- **File-based POST bodies** — use `@filename` syntax to load payloads from disk
- **Configurable concurrency, timeout, and request count**
- **Zero system dependencies** — statically compiled with rustls (no OpenSSL needed)

## Installation

### Prerequisites

Install the Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build & Install

```bash
git clone https://github.com/YOUR_USERNAME/loadmaxx.git
cd loadmaxx
cargo install --path .
```

This compiles an optimized release build and installs the `loadmaxx` binary to `~/.cargo/bin/`, which should already be on your PATH.

### Verify

```bash
loadmaxx --help
```

## Usage

### Options

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| `-u` | `--url` | (required) | Target URL to test |
| `-n` | `--requests` | 100 | Total number of requests to send |
| `-c` | `--concurrency` | 10 | Number of concurrent workers |
| `-t` | `--timeout` | 30 | Request timeout in seconds |
| `-o` | `--output` | loadtest_log.csv | CSV log file path |
| `-m` | `--method` | GET | HTTP method (GET or POST) |
| `-b` | `--body` | (none) | POST body (string or @filename) |
| | `--content-type` | application/json | Content-Type header for POST |
| | `--http2` | false | Force HTTP/2 prior knowledge |

### GET Requests

GET is the default method. Use it to test page loads, static assets, API reads, and health checks.

```bash
# Basic test: 100 requests, 10 concurrent (defaults)
loadmaxx --url https://your-site.com

# Heavy load: 5,000 requests, 100 concurrent
loadmaxx --url https://your-site.com -n 5000 -c 100

# Quick smoke test with short timeout
loadmaxx --url https://your-site.com -n 20 -c 5 -t 5

# Test an API endpoint
loadmaxx --url https://api.your-site.com/v1/users -n 500 -c 25

# Custom CSV output path
loadmaxx --url https://your-site.com -n 1000 -c 50 -o results.csv
```

### POST Requests

Use `-m POST` to test API endpoints that accept data — login flows, form submissions, webhooks, etc.

```bash
# POST JSON payload
loadmaxx --url https://api.example.com/login \
  -m POST \
  -b '{"username":"test","password":"test123"}' \
  -n 500 -c 20

# POST form data
loadmaxx --url https://api.example.com/submit \
  -m POST \
  -b 'username=test&password=test123' \
  --content-type "application/x-www-form-urlencoded" \
  -n 500 -c 20

# POST XML
loadmaxx --url https://api.example.com/soap \
  -m POST \
  -b '<request><action>test</action></request>' \
  --content-type "application/xml" \
  -n 200 -c 10

# POST with body loaded from a file
loadmaxx --url https://api.example.com/data \
  -m POST \
  -b @payload.json \
  -n 1000 -c 50

# POST with empty body (health check / trigger style)
loadmaxx --url https://api.example.com/webhook \
  -m POST \
  -n 100 -c 10
```

The `@filename` syntax works like curl — prefix a file path with `@` and LoadMaxx reads the file contents as the request body.

### HTTP/2

By default, LoadMaxx auto-negotiates the protocol via ALPN during the TLS handshake. If the server supports HTTP/2 over HTTPS, it will be used automatically.

Use `--http2` to force HTTP/2 prior knowledge — this skips negotiation and speaks HTTP/2 immediately. Useful for testing h2c (HTTP/2 cleartext) or benchmarking HTTP/2 vs HTTP/1.1.

```bash
# Auto-negotiate (default) — uses HTTP/2 on HTTPS if server supports it
loadmaxx --url https://your-site.com -n 1000 -c 50

# Force HTTP/2
loadmaxx --url https://your-site.com -n 1000 -c 50 --http2

# Force HTTP/2 on a plaintext target (h2c)
loadmaxx --url http://your-site.com:8080 -n 1000 -c 50 --http2
```

### Testing with httpbin.org

[httpbin.org](https://httpbin.org) is a free public API that echoes back whatever you send. Useful for verifying your setup before pointing at a real target.

```bash
# Test GET
loadmaxx --url https://httpbin.org/get -n 50 -c 5

# Test POST
loadmaxx --url https://httpbin.org/post \
  -m POST \
  -b '{"test":"loadmaxx"}' \
  -n 50 -c 5
```

## Output

### Terminal Report

```
🚀 LoadMaxx
  Target:       https://example.com
  Method:       GET
  Protocol:     Auto (HTTP/2 via ALPN on HTTPS, HTTP/1.1 on HTTP)
  Requests:     1000
  Concurrency:  50
  Timeout:      30s
  Log file:     loadtest_log.csv

═══════════════════════════════════════
         LOAD TEST RESULTS
═══════════════════════════════════════

  Summary
  Total requests:    1000
  Successful:        998
  Failed:            2
  Total time:        4.32s
  Requests/sec:      231.48

  Latency
  Average:           42.15 ms
  Min:               12.03 ms
  Max:               312.44 ms
  P50:               38.21 ms
  P90:               78.55 ms
  P99:               210.33 ms

  Status Codes
  200:               998 responses
  503:               2 responses

═══════════════════════════════════════

  ✓ Logged to loadtest_log.csv
```

### CSV Log

Each request is logged with full detail:

```csv
request_number,timestamp,status,latency_ms,success,error
1,2026-03-03 14:22:01.234,200,45.23,true,""
2,2026-03-03 14:22:01.238,200,52.11,true,""
3,2026-03-03 14:22:01.241,503,30.05,false,""
```

## Project Structure

```
loadmaxx/
├── Cargo.toml        # Dependencies and project config
├── .gitignore        # Excludes /target from git
└── src/
    ├── main.rs       # CLI parsing, validation, and orchestration
    ├── loader.rs     # HTTP client, GET/POST execution, HTTP/2 support
    └── stats.rs      # Results aggregation, reporting, and CSV export
```

## License

MIT
