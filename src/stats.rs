use crate::loader::RequestResult;
use colored::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

#[derive(Debug)]
pub struct Stats {
    pub results: Vec<RequestResult>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            results: Vec::new(),
        }
    }

    pub fn report(&self, total_duration: Duration) {
        let total = self.results.len();
        let successes = self.results.iter().filter(|r| r.success).count();
        let failures = total - successes;

        let durations: Vec<f64> = self
            .results
            .iter()
            .map(|r| r.duration.as_secs_f64() * 1000.0)
            .collect();

        let avg = durations.iter().sum::<f64>() / durations.len() as f64;
        let min = durations.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = durations.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let mut sorted = durations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = percentile(&sorted, 50.0);
        let p90 = percentile(&sorted, 90.0);
        let p99 = percentile(&sorted, 99.0);

        let rps = total as f64 / total_duration.as_secs_f64();

        let mut status_counts: HashMap<u16, usize> = HashMap::new();
        for r in &self.results {
            if let Some(code) = r.status {
                *status_counts.entry(code).or_insert(0) += 1;
            }
        }

        println!(
            "\n{}",
            "═══════════════════════════════════════".bright_cyan()
        );
        println!(
            "{}",
            "         LOAD TEST RESULTS".bright_cyan().bold()
        );
        println!(
            "{}",
            "═══════════════════════════════════════".bright_cyan()
        );

        println!("\n{}", "  Summary".bold().underline());
        println!(
            "  Total requests:    {}",
            total.to_string().white().bold()
        );
        println!(
            "  Successful:        {}",
            successes.to_string().green().bold()
        );
        println!(
            "  Failed:            {}",
            if failures > 0 {
                failures.to_string().red().bold()
            } else {
                failures.to_string().green().bold()
            }
        );
        println!("  Total time:        {:.2}s", total_duration.as_secs_f64());
        println!(
            "  Requests/sec:      {}",
            format!("{:.2}", rps).yellow().bold()
        );

        println!("\n{}", "  Latency".bold().underline());
        println!("  Average:           {:.2} ms", avg);
        println!("  Min:               {:.2} ms", min);
        println!("  Max:               {:.2} ms", max);
        println!("  P50:               {:.2} ms", p50);
        println!("  P90:               {:.2} ms", p90);
        println!("  P99:               {:.2} ms", p99);

        if !status_counts.is_empty() {
            println!("\n{}", "  Status Codes".bold().underline());
            for (code, count) in &status_counts {
                let colored_code = match code {
                    200..=299 => code.to_string().green(),
                    300..=399 => code.to_string().yellow(),
                    _ => code.to_string().red(),
                };
                println!("  {}:               {} responses", colored_code, count);
            }
        }

        // Connection-level errors (no response received)
        let errors: Vec<&str> = self
            .results
            .iter()
            .filter_map(|r| r.error.as_deref())
            .collect();
        if !errors.is_empty() {
            println!("\n{}", "  Errors".bold().underline().red());
            let mut error_counts: HashMap<&str, usize> = HashMap::new();
            for e in &errors {
                *error_counts.entry(e).or_insert(0) += 1;
            }
            for (err, count) in &error_counts {
                println!("  {} × {}", count.to_string().red(), err);
            }
        }

        // Non-200 response bodies (show first unique per status code)
        let non_success: Vec<&RequestResult> = self
            .results
            .iter()
            .filter(|r| !r.success && r.response_body.is_some())
            .collect();
        if !non_success.is_empty() {
            println!(
                "\n{}",
                "  Response Bodies (non-2xx)".bold().underline().red()
            );
            let mut seen_codes: HashMap<u16, bool> = HashMap::new();
            for r in &non_success {
                if let Some(code) = r.status {
                    if seen_codes.contains_key(&code) {
                        continue;
                    }
                    seen_codes.insert(code, true);
                    if let Some(body) = &r.response_body {
                        let preview: String = body
                            .chars()
                            .take(200)
                            .collect::<String>()
                            .replace('\n', " ")
                            .replace('\r', "");
                        println!(
                            "  {} → {}",
                            code.to_string().red().bold(),
                            preview.dimmed()
                        );
                    }
                }
            }
        }

        println!(
            "\n{}",
            "═══════════════════════════════════════".bright_cyan()
        );
    }

    pub fn write_csv(&self, path: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        writeln!(
            file,
            "request_number,timestamp,status,latency_ms,success,error,response_body"
        )?;

        let mut sorted = self.results.clone();
        sorted.sort_by_key(|r| r.request_number);

        for r in &sorted {
            let body = r
                .response_body
                .as_deref()
                .unwrap_or("")
                .replace('"', "'")
                .replace('\n', " ")
                .replace('\r', "");

            writeln!(
                file,
                "{},{},{},{:.2},{},\"{}\",\"{}\"",
                r.request_number,
                r.timestamp,
                r.status.map_or("N/A".to_string(), |s| s.to_string()),
                r.duration.as_secs_f64() * 1000.0,
                r.success,
                sanitize_csv(r.error.as_deref().unwrap_or("")),
                sanitize_csv(&body)
            )?;
        }
        Ok(())
    }
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (pct / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn sanitize_csv(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.starts_with('=')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('@')
    {
        format!("'{}", trimmed)
    } else {
        trimmed.to_string()
    }
}
