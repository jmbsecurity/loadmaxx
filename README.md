# LoadMaxx

A blazing-fast CLI HTTP load testing tool built in Rust. Designed to generate massive concurrent load with minimal resource overhead.


## Features

- **High-concurrency async I/O** — powered by tokio and reqwest
- **Detailed latency stats** — avg, min, max, P50, P90, P99
- **CSV request logging** — per-request timestamp, status code, latency, and errors
- **Progress bar** — real-time completion tracking
- **Status code breakdown** — color-coded 2xx/3xx/4xx/5xx summary
- **Error aggregation** — grouped error counts for quick diagnosis
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

```bash
# Basic test: 100 requests, 10 concurrent (defaults)
loadmaxx --url https://your-site.com

# Heavy load: 5,000 requests, 100 concurrent
loadmaxx --url https://your-site.com -n 5000 -c 100

# Quick smoke test with short timeout
loadmaxx --url https://your-site.com -n 20 -c 5 -t 5

# Custom CSV output path
loadmaxx --url https://your-site.com -n 1000 -c 50 -o results.csv
```

### Options

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| `-u` | `--url` | (required) | Target URL to test |
| `-n` | `--requests` | 100 | Total number of requests to send |
| `-c` | `--concurrency` | 10 | Number of concurrent workers |
| `-t` | `--timeout` | 30 | Request timeout in seconds |
| `-o` | `--output` | loadtest_log.csv | CSV log file path |

## Output

### Terminal Report

```
🚀 Load Tester
  Target:       https://example.com
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
└── src/
    ├── main.rs       # CLI parsing and orchestration
    ├── loader.rs     # HTTP client and request execution
    └── stats.rs      # Results aggregation, reporting, and CSV export
```

## License

MIT
