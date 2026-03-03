mod loader;
mod stats;

use clap::Parser;
use colored::*;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;

/// A fast CLI HTTP load testing tool
#[derive(Parser, Debug)]
#[command(name = "loadtester", version, about)]
struct Args {
    /// Target URL to test
    #[arg(short, long)]
    url: String,

    /// Total number of requests to send
    #[arg(short = 'n', long, default_value_t = 100)]
    requests: u32,

    /// Number of concurrent workers
    #[arg(short, long, default_value_t = 10)]
    concurrency: u32,

    /// Request timeout in seconds
    #[arg(short, long, default_value_t = 30)]
    timeout: u64,

    /// Output CSV log file path
    #[arg(short, long, default_value = "loadtest_log.csv")]
    output: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("{}", "\n🚀 Load Tester".bright_cyan().bold());
    println!("  Target:       {}", args.url.white().bold());
    println!("  Requests:     {}", args.requests);
    println!("  Concurrency:  {}", args.concurrency);
    println!("  Timeout:      {}s", args.timeout);
    println!("  Log file:     {}", args.output.white());
    println!("{}", "\nStarting...\n".yellow());

    let client = loader::build_client(args.timeout, args.concurrency);
    let stats = Arc::new(Mutex::new(stats::Stats::new()));
    let completed = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let total_start = Instant::now();

    let mut tasks = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(args.concurrency as usize));

    for i in 0..args.requests {
        let client = client.clone();
        let url = args.url.clone();
        let stats = Arc::clone(&stats);
        let completed = Arc::clone(&completed);
        let semaphore = Arc::clone(&semaphore);
        let total = args.requests;

        tasks.spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = loader::send_request(&client, &url, i + 1).await;

            let count = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let step = (total / 10).max(1);
            if count % step == 0 || count == total {
                let pct = (count as f64 / total as f64 * 100.0) as u32;
                let bar_len = 30;
                let filled = (pct as usize * bar_len) / 100;
                let bar: String = "█".repeat(filled) + &"░".repeat(bar_len - filled);
                print!("\r  [{}] {}% ({}/{})", bar.bright_cyan(), pct, count, total);
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

    match locked_stats.write_csv(&args.output) {
        Ok(_) => println!("\n  {} Logged to {}", "✓".green().bold(), args.output.white().bold()),
        Err(e) => eprintln!("\n  {} Failed to write CSV: {}", "✗".red().bold(), e),
    }
}
