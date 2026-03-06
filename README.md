# LoadMaxx

A blazing-fast CLI HTTP load testing tool built in Rust. Designed to generate massive concurrent load with minimal resource overhead.

## Features

- **GET and POST support** — test static pages, APIs, login flows, and form submissions
- **HTTP/2 support** — auto-negotiates on HTTPS, or force with `--http2`
- **Response body capture** — see exactly what the server returns on non-2xx responses
- **High-concurrency async I/O** — powered by tokio and reqwest
- **Detailed latency stats** — avg, min, max, P50, P90, P99
- **Multi-URL testing** — test multiple endpoints at once with round-robin distribution
- **CSV and JSON logging** — per-request timestamp, URL, status, latency, errors, and response bodies
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
git clone https://github.com/jmbsecurity/loadmaxx.git
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
| `-u` | `--url` | (required) | Target URL(s) to test (can specify multiple) |
| `-n` | `--requests` | 100 | Total number of requests to send |
| `-c` | `--concurrency` | 10 | Number of concurrent workers |
| `-t` | `--timeout` | 30 | Request timeout in seconds |
| `-o` | `--output` | loadtest_log | Output log file path (extension auto-added from format) |
| `-f` | `--format` | csv | Output format (`csv` or `json`) |
| `-m` | `--method` | GET | HTTP method (GET or POST) |
| `-b` | `--body` | (none) | POST body (string or @filename) |
| | `--url-file` | (none) | File containing URLs to test (one per line) |
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

### Multiple URLs

LoadMaxx can test multiple endpoints at once. Requests are distributed across URLs in round-robin order — request 1 goes to URL 1, request 2 to URL 2, and so on, cycling back to the start.

```bash
# Test two endpoints, 1000 requests split evenly across both
loadmaxx -u https://api.example.com/users -u https://api.example.com/products \
  -n 1000 -c 50

# Test three API routes with POST
loadmaxx -u https://api.example.com/v1/search \
  -u https://api.example.com/v1/recommend \
  -u https://api.example.com/v1/trending \
  -m POST \
  -b '{"query":"test"}' \
  -n 3000 -c 30

# Combine multiple URLs with JSON logging
loadmaxx -u https://api.example.com/health \
  -u https://api.example.com/status \
  -n 500 -c 20 -f json
```

You can also pass multiple URLs in a single `-u` flag:

```bash
loadmaxx -u https://example.com/a https://example.com/b https://example.com/c \
  -n 600 -c 30
```

Or load URLs from a text file with `--url-file`:

```bash
# urls.txt
https://api.example.com/v1/users
https://api.example.com/v1/products
https://api.example.com/v1/orders
# comments and blank lines are ignored
```

```bash
loadmaxx --url-file urls.txt -n 3000 -c 50

# Combine file and inline URLs
loadmaxx --url-file urls.txt -u https://api.example.com/v1/health -n 1000 -c 25
```

### Output Format

Use `-f` to choose between CSV (default) and JSON logging:

```bash
# CSV output (default)
loadmaxx --url https://example.com -n 100 -f csv

# JSON output
loadmaxx --url https://example.com -n 100 -f json

# JSON output with custom file path
loadmaxx --url https://example.com -n 100 -f json -o results.json
```

The output file extension is added automatically based on the format if you don't include one (e.g., `loadtest_log.csv` or `loadtest_log.json`).

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
  Format:       csv
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
  429:               2 responses

  Response Bodies (non-2xx)
  429 → <html><head><title>429 Too Many Requests</title></head>...

═══════════════════════════════════════

  ✓ Logged to loadtest_log.csv
```

### CSV Log

Each request is logged with full detail including the target URL and response bodies for non-2xx responses:

```csv
request_number,url,timestamp,status,latency_ms,success,error,response_body
1,"https://api.example.com/users",2026-03-03 14:22:01.234,200,45.23,true,"",""
2,"https://api.example.com/products",2026-03-03 14:22:01.238,429,52.11,false,"","<html>..."
3,"https://api.example.com/users",2026-03-03 14:22:01.241,200,30.05,true,"",""
```

### JSON Log

Use `-f json` for structured JSON output:

```json
[
  {
    "request_number": 1,
    "url": "https://api.example.com/users",
    "timestamp": "2026-03-03 14:22:01.234",
    "status": 200,
    "latency_ms": 45.23,
    "success": true,
    "error": "",
    "response_body": ""
  },
  {
    "request_number": 2,
    "url": "https://api.example.com/products",
    "timestamp": "2026-03-03 14:22:01.238",
    "status": 429,
    "latency_ms": 52.11,
    "success": false,
    "error": "",
    "response_body": "<html>..."
  }
]
```

## Project Structure

```
loadmaxx/
├── Cargo.toml        # Dependencies and project config
├── .gitignore        # Excludes /target from git
└── src/
    ├── main.rs       # CLI parsing, validation, and orchestration
    ├── loader.rs     # HTTP client, GET/POST execution, HTTP/2, response capture
    └── stats.rs      # Results aggregation, reporting, and CSV/JSON export
```

## License

MIT
