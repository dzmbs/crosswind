use std::io::{self, IsTerminal};

use comfy_table::{Cell, Color, ContentArrangement, Table};
use serde_json::{Value, json};

use crate::model::SearchResult;

pub fn is_tty() -> bool {
    io::stdout().is_terminal()
}

pub fn format_duration(minutes: i32) -> String {
    let h = minutes / 60;
    let m = minutes % 60;
    if h > 0 && m > 0 {
        format!("{h}h {m}m")
    } else if h > 0 {
        format!("{h}h")
    } else {
        format!("{m}m")
    }
}

fn stops_label(stops: i32) -> String {
    match stops {
        0 => "Nonstop".into(),
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

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);
    table.set_header(vec![
        Cell::new("#"),
        Cell::new("Airlines"),
        Cell::new("Route"),
        Cell::new("Depart"),
        Cell::new("Arrive"),
        Cell::new("Duration"),
        Cell::new("Stops"),
        Cell::new("Price"),
    ]);

    for (i, flight) in result.flights.iter().enumerate() {
        let first_seg = &flight.segments[0];
        let last_seg = flight.segments.last().unwrap();

        let route = format!("{}→{}", first_seg.from_code, last_seg.to_code);
        let airlines_str = flight.airlines.join(", ");

        let depart = format!("{} {}", first_seg.depart_date, first_seg.depart_time);
        let arrive = format!("{} {}", last_seg.arrive_date, last_seg.arrive_time);

        let price_str = if flight.price > 0 {
            format!("{sym}{}", flight.price)
        } else {
            "—".into()
        };

        let stops = stops_label(flight.stops);

        let mut row = vec![
            Cell::new(i + 1),
            Cell::new(&airlines_str),
            Cell::new(&route),
            Cell::new(&depart),
            Cell::new(&arrive),
            Cell::new(format_duration(flight.duration_minutes)),
            Cell::new(&stops),
            Cell::new(&price_str),
        ];

        if flight.is_best {
            for cell in &mut row {
                *cell = cell.clone().fg(Color::Green);
            }
        }

        table.add_row(row);
    }

    println!("{table}");
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
