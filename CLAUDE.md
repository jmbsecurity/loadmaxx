# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LoadMaxx is a CLI HTTP load testing tool built in Rust. It sends concurrent GET/POST requests to a target URL and reports latency statistics (avg, min, max, P50, P90, P99), status code breakdowns, error aggregation, and per-request CSV logs.

## Build & Run

```bash
cargo build --release          # Build optimized binary
cargo install --path .         # Install to ~/.cargo/bin/
cargo run -- --url <URL> [options]  # Run directly during development
```

Rust edition 2024. No OpenSSL — uses rustls via reqwest.

## Architecture

Three source files in `src/`:

- **main.rs** — CLI argument parsing (clap derive), input validation, async orchestration. Spawns concurrent tasks via `tokio::JoinSet` with a `Semaphore` for concurrency control. Collects results into shared `Arc<Mutex<Stats>>`.
- **loader.rs** — HTTP client construction (`build_client`) and request execution (`send_request`). Handles GET/POST dispatch, HTTP/2 prior knowledge, and captures response bodies for non-2xx responses (truncated to 512 chars via `MAX_BODY_CAPTURE`).
- **stats.rs** — `Stats` struct that aggregates `RequestResult` entries. `report()` prints the terminal summary with colored output. `write_csv()` exports results with CSV injection protection via `sanitize_csv()`.

Data flow: `main` spawns tasks → each calls `loader::send_request` → pushes `RequestResult` into `Stats` → after all tasks complete, `Stats::report()` and `Stats::write_csv()` produce output.

## Key Dependencies

- **tokio** (async runtime, full features)
- **reqwest** (HTTP client, rustls-tls + http2 features, no default features)
- **clap** (CLI parsing, derive feature)
- **colored** (terminal colors)
- **chrono** (timestamps)

## Conventions

- Max requests capped at 1,000,000 (`MAX_REQUESTS` in main.rs) to prevent self-DoS
- POST body supports `@filename` syntax (like curl) to load payloads from disk
- CSV output sanitizes fields starting with `=`, `+`, `-`, `@` to prevent formula injection
