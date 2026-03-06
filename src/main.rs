mod loader;
mod stats;

use clap::Parser;
use colored::*;
use loader::HttpMethod;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

const MAX_REQUESTS: u32 = 1_000_000;

/// A fast CLI HTTP load testing tool
#[derive(Parser, Debug)]
#[command(name = "loadmaxx", version, about)]
struct Args {
    /// Target URL(s) to test (can specify multiple)
    #[arg(short, long, num_args = 1..)]
    url: Vec<String>,

    /// Total number of requests to send
    #[arg(short = 'n', long, default_value_t = 100)]
    requests: u32,

    /// Number of concurrent workers
    #[arg(short, long, default_value_t = 10)]
    concurrency: u32,

    /// Request timeout in seconds
    #[arg(short, long, default_value_t = 30)]
    timeout: u64,

    /// Output log file path
    #[arg(short, long, default_value = "loadtest_log")]
    output: String,

    /// Output format (csv or json)
    #[arg(short, long, default_value = "csv")]
    format: String,

    /// HTTP method (GET or POST)
    #[arg(short, long, default_value = "GET")]
    method: String,

    /// POST request body (string or @filename to read from file)
    #[arg(short, long)]
    body: Option<String>,

    /// Content-Type header for POST requests
    #[arg(long, default_value = "application/json")]
    content_type: String,

    /// Force HTTP/2 prior knowledge (skip ALPN negotiation)
    #[arg(long, default_value_t = false)]
    http2: bool,
}

fn validate_urls(urls: &[String]) -> Result<(), String> {
    if urls.is_empty() {
        return Err("At least one URL is required".to_string());
    }
    for url in urls {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(format!("URL must start with http:// or https://: {}", url));
        }

        let lower = url.to_lowercase();
        if lower.contains("localhost") || lower.contains("127.0.0.1") || lower.contains("[::1]") {
            eprintln!(
                "  {} Targeting localhost/loopback address: {}",
                "⚠".yellow().bold(),
                url
            );
        }
    }
    Ok(())
}

fn validate_format(format: &str) -> Result<(), String> {
    match format {
        "csv" | "json" => Ok(()),
        other => Err(format!("Unsupported output format '{}'. Use 'csv' or 'json'", other)),
    }
}

fn parse_method(args: &Args) -> HttpMethod {
    match args.method.to_uppercase().as_str() {
        "GET" => HttpMethod::Get,
        "POST" => {
            let body = match &args.body {
                Some(b) if b.starts_with('@') => {
                    let path = &b[1..];
                    std::fs::read_to_string(path).unwrap_or_else(|e| {
                        eprintln!("Error reading body file '{}': {}", path, e);
                        std::process::exit(1);
                    })
                }
                Some(b) => b.clone(),
                None => String::new(),
            };
            HttpMethod::Post {
                body,
                content_type: args.content_type.clone(),
            }
        }
        other => {
            eprintln!("Unsupported HTTP method: {}", other);
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Validate inputs
    if let Err(e) = validate_urls(&args.url) {
        eprintln!("  {} {}", "✗".red().bold(), e);
        std::process::exit(1);
    }

    if let Err(e) = validate_format(&args.format) {
        eprintln!("  {} {}", "✗".red().bold(), e);
        std::process::exit(1);
    }

    let output_path = if args.output.contains('.') {
        args.output.clone()
    } else {
        format!("{}.{}", args.output, args.format)
    };

    if args.requests > MAX_REQUESTS {
        eprintln!(
            "  {} Max requests capped at {} to prevent self-DoS",
            "✗".red().bold(),
            MAX_REQUESTS
        );
        std::process::exit(1);
    }

    let method = parse_method(&args);

    let method_display = match &method {
        HttpMethod::Get => "GET".to_string(),
        HttpMethod::Post { body, content_type } => {
            format!("POST ({}, {} bytes)", content_type, body.len())
        }
    };

    let protocol_display = if args.http2 {
        "HTTP/2 (forced)"
    } else {
        "Auto (HTTP/2 via ALPN on HTTPS, HTTP/1.1 on HTTP)"
    };

    println!("{}", "\n🚀 LoadMaxx".bright_cyan().bold());
    if args.url.len() == 1 {
        println!("  Target:       {}", args.url[0].white().bold());
    } else {
        println!("  Targets:      {} URLs (round-robin)", args.url.len());
        for (i, u) in args.url.iter().enumerate() {
            println!("    [{}] {}", i + 1, u.white().bold());
        }
    }
    println!("  Method:       {}", method_display.white().bold());
    println!("  Protocol:     {}", protocol_display.white().bold());
    println!("  Requests:     {}", args.requests);
    println!("  Concurrency:  {}", args.concurrency);
    println!("  Timeout:      {}s", args.timeout);
    println!("  Format:       {}", args.format.white());
    println!("  Log file:     {}", output_path.white());
    println!("{}", "\nStarting...\n".yellow());

    let client = loader::build_client(args.timeout, args.concurrency, args.http2);
    let stats = Arc::new(Mutex::new(stats::Stats::new()));
    let completed = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let total_start = Instant::now();

    let mut tasks = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(args.concurrency as usize));
    let method = Arc::new(method);
    let urls = Arc::new(args.url);

    for i in 0..args.requests {
        let client = client.clone();
        let urls = Arc::clone(&urls);
        let stats = Arc::clone(&stats);
        let completed = Arc::clone(&completed);
        let semaphore = Arc::clone(&semaphore);
        let method = Arc::clone(&method);
        let total = args.requests;

        tasks.spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let url = &urls[i as usize % urls.len()];
            let result = loader::send_request(&client, url, i + 1, &method).await;

            let count = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let step = (total / 10).max(1);
            if count % step == 0 || count == total {
                let pct = (count as f64 / total as f64 * 100.0) as u32;
                let bar_len = 30;
                let filled = (pct as usize * bar_len) / 100;
                let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
                print!(
                    "\r  [{}] {}% ({}/{})",
                    bar.bright_cyan(),
                    pct,
                    count,
                    total
                );
                use std::io::Write;
                std::io::stdout().flush().ok();
            }

            stats.lock().await.results.push(result);
        });
    }

    while tasks.join_next().await.is_some() {}

    let total_duration = total_start.elapsed();
    println!();

    let locked_stats = stats.lock().await;
    locked_stats.report(total_duration);

    let write_result = match args.format.as_str() {
        "json" => locked_stats.write_json(&output_path),
        _ => locked_stats.write_csv(&output_path),
    };

    match write_result {
        Ok(_) => println!(
            "\n  {} Logged to {}",
            "✓".green().bold(),
            output_path.white().bold()
        ),
        Err(e) => eprintln!(
            "\n  {} Failed to write log: {}",
            "✗".red().bold(),
            e
        ),
    }
}
