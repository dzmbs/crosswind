use std::io::{self, IsTerminal, Write};

use serde_json::{Value, json};

use crate::model::SearchResult;

#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Pretty,
    Json,
    Ndjson,
    Quiet,
}

impl OutputFormat {
    pub fn detect(json: bool, output: Option<&str>, quiet: bool) -> Result<Self, String> {
        if quiet {
            return Ok(Self::Quiet);
        }
        if let Some(fmt) = output {
            return match fmt {
                "json" => Ok(Self::Json),
                "ndjson" | "jsonl" => Ok(Self::Ndjson),
                "pretty" => Ok(Self::Pretty),
                "quiet" => Ok(Self::Quiet),
                _ => Err(format!(
                    "unknown output format '{fmt}', use pretty, json, ndjson, or quiet"
                )),
            };
        }
        if json {
            return Ok(Self::Json);
        }
        if io::stdout().is_terminal() {
            Ok(Self::Pretty)
        } else {
            Ok(Self::Json)
        }
    }

    pub fn is_machine(&self) -> bool {
        matches!(self, Self::Json | Self::Ndjson)
    }
}

pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

pub const DIM: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";

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

pub fn print_table(result: &SearchResult, currency: &str) {
    let sym = currency_symbol(currency);

    if result.flights.is_empty() {
        eprintln!("No flights found.");
        return;
    }

    eprintln!(
        "{DIM}  #  Price    Airlines                  Route       Depart       Arrive       Dur       Stops{RESET}"
    );
    eprintln!(
        "{DIM}  -  -------  ------------------------  ----------  -----------  -----------  --------  -------{RESET}"
    );

    for (i, flight) in result.flights.iter().enumerate() {
        let first_seg = &flight.segments[0];
        let last_seg = flight.segments.last().unwrap();

        let route = format!("{}>{}", first_seg.from_code, last_seg.to_code);
        let airlines_str = flight.airlines.join(", ");
        let depart = format!(
            "{} {}",
            first_seg
                .depart_date
                .get(5..)
                .unwrap_or(&first_seg.depart_date),
            first_seg.depart_time
        );
        let arrive = format!(
            "{} {}",
            last_seg
                .arrive_date
                .get(5..)
                .unwrap_or(&last_seg.arrive_date),
            last_seg.arrive_time
        );
        let price_str = if flight.price > 0 {
            format!("{sym}{}", flight.price)
        } else {
            "-".into()
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
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 1).collect();
        format!("{truncated}…")
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
    let min_price = result
        .flights
        .iter()
        .filter(|f| f.price > 0)
        .map(|f| f.price)
        .min()
        .unwrap_or(0);
    println!("{sym}{min_price}");
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
