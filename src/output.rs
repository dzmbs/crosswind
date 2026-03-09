use std::io::{self, IsTerminal, Write};

use serde_json::{Value, json};

use crate::model::SearchResult;

/// Output format, matching heat-cli/hlz conventions.
#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// Styled table for humans (TTY)
    Pretty,
    /// JSON envelope (piped or --json)
    Json,
    /// Newline-delimited JSON, one flight per line
    Ndjson,
    /// Minimal output (just count or first price)
    Quiet,
}

impl OutputFormat {
    /// Auto-detect format from flags and TTY state.
    /// Priority: explicit flags > TTY detection.
    pub fn detect(json: bool, output: Option<&str>, quiet: bool) -> Self {
        if quiet {
            return Self::Quiet;
        }
        if let Some(fmt) = output {
            return match fmt {
                "json" => Self::Json,
                "ndjson" | "jsonl" => Self::Ndjson,
                "pretty" => Self::Pretty,
                "quiet" => Self::Quiet,
                _ => Self::Json,
            };
        }
        if json {
            return Self::Json;
        }
        if io::stdout().is_terminal() {
            Self::Pretty
        } else {
            Self::Json
        }
    }

    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json | Self::Ndjson)
    }
}

pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

pub fn format_duration(minutes: i32) -> String {
    let h = minutes / 60;
    let m = minutes % 60;
    if h > 0 && m > 0 {
        format!("{h}h{m:02}m")
    } else if h > 0 {
        format!("{h}h")
    } else {
        format!("{m}m")
    }
}

fn stops_label(stops: i32) -> String {
    match stops {
        0 => "nonstop".into(),
        1 => "1 stop".into(),
        n => format!("{n} stops"),
    }
}

fn currency_symbol(currency: &str) -> &str {
    match currency {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" | "CNY" => "¥",
        "RSD" => "RSD ",
        _ => "",
    }
}

const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";

pub fn print_table(result: &SearchResult, currency: &str) {
    let sym = currency_symbol(currency);

    if result.flights.is_empty() {
        eprintln!("No flights found.");
        return;
    }

    // header to stderr (diagnostics)
    eprintln!(
        "{DIM}  #  Price    Airlines                  Route       Depart       Arrive       Dur       Stops{RESET}"
    );
    eprintln!(
        "{DIM}  ─  ───────  ────────────────────────  ──────────  ───────────  ───────────  ────────  ───────{RESET}"
    );

    for (i, flight) in result.flights.iter().enumerate() {
        let first_seg = &flight.segments[0];
        let last_seg = flight.segments.last().unwrap();

        let route = format!("{}→{}", first_seg.from_code, last_seg.to_code);
        let airlines_str = flight.airlines.join(", ");
        let depart = format!("{} {}", &first_seg.depart_date[5..], first_seg.depart_time);
        let arrive = format!("{} {}", &last_seg.arrive_date[5..], last_seg.arrive_time);
        let price_str = if flight.price > 0 {
            format!("{sym}{}", flight.price)
        } else {
            "—".into()
        };
        let dur = format_duration(flight.duration_minutes);
        let stops = stops_label(flight.stops);

        let line = format!(
            "{:>3}  {:<7}  {:<24}  {:<10}  {:<11}  {:<11}  {:>8}  {}",
            i + 1,
            price_str,
            truncate(&airlines_str, 24),
            route,
            depart,
            arrive,
            dur,
            stops,
        );

        if flight.is_best {
            println!("{GREEN}{}{RESET}", line.trim_end());
        } else {
            println!("{}", line.trim_end());
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

pub fn to_json_envelope(result: &SearchResult, cmd: &str, timing_ms: u64) -> Value {
    json!({
        "v": 1,
        "status": "ok",
        "cmd": cmd,
        "data": result,
        "timing_ms": timing_ms,
    })
}

pub fn print_json(result: &SearchResult, cmd: &str, timing_ms: u64) {
    let envelope = to_json_envelope(result, cmd, timing_ms);
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}

pub fn print_ndjson(result: &SearchResult) {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for flight in &result.flights {
        let _ = serde_json::to_writer(&mut out, flight);
        let _ = out.write_all(b"\n");
    }
}

pub fn print_quiet(result: &SearchResult, currency: &str) {
    if result.flights.is_empty() {
        return;
    }
    let sym = currency_symbol(currency);
    let cheapest = &result.flights[0];
    println!("{sym}{}", cheapest.price);
}

pub fn print_error_json(err: &crate::error::CrosswindError, cmd: &str, timing_ms: u64) {
    let envelope = err.to_json(cmd, timing_ms);
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}

pub fn print_error_text(err: &crate::error::CrosswindError) {
    eprintln!("error: {err}");
    if let Some(hint) = err.hint() {
        eprintln!("hint: {hint}");
    }
}
