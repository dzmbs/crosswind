use std::io::{self, IsTerminal};

use serde_json::{Value, json};

use crate::model::SearchResult;

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

    // header
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
        let depart = format!(
            "{} {}",
            first_seg.depart_date.chars().skip(5).collect::<String>(),
            first_seg.depart_time
        );
        let arrive = format!(
            "{} {}",
            last_seg.arrive_date.chars().skip(5).collect::<String>(),
            last_seg.arrive_time
        );
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

    eprintln!();
    eprintln!("{DIM}{} flights found{RESET}", result.flights.len());
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

pub fn print_error_json(err: &crate::error::CrosswindError) {
    let envelope = err.to_json();
    println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
}
