use std::process;
use std::time::Instant;

use clap::Parser;

use crosswind::date::parse_date;
use crosswind::error::CrosswindError;
use crosswind::model::{Flight, SearchResult};
use crosswind::output::{self, DIM, OutputFormat, RESET};
use crosswind::query::QueryParams;
use crosswind::query::proto::Cabin;

/// Google Flights from your terminal
#[derive(Parser)]
#[command(name = "crosswind", version, about)]
struct Cli {
    /// Origin airport code (e.g. BEG, JFK, LAX)
    origin: String,

    /// Destination airport code(s), comma-separated (e.g. JFK or JFK,LHR)
    #[arg(name = "destination")]
    destinations: String,

    /// Departure date (apr1, 4/1, +7, tomorrow, 2026-04-01)
    date: String,

    /// Return date for round-trip
    #[arg(short, long)]
    ret: Option<String>,

    /// Cabin class: economy, premium, business, first
    #[arg(short, long, default_value = "economy")]
    cabin: String,

    /// Number of adult passengers (1-9)
    #[arg(short = 'p', long, default_value = "1")]
    adults: u32,

    /// Maximum stops (0 = nonstop only)
    #[arg(long)]
    max_stops: Option<i32>,

    /// Nonstop flights only (shorthand for --max-stops 0)
    #[arg(long)]
    nonstop: bool,

    /// Maximum flight duration in minutes
    #[arg(long)]
    max_duration: Option<i32>,

    /// Maximum price (in --currency units)
    #[arg(long)]
    max_price: Option<i64>,

    /// Sort by: price, duration, stops, departure
    #[arg(long, default_value = "price")]
    sort: String,

    /// Show only the top N results
    #[arg(long)]
    top: Option<usize>,

    /// Currency code (USD, EUR, GBP, etc.)
    #[arg(long, default_value = "USD")]
    currency: String,

    /// Language code
    #[arg(long, default_value = "en")]
    lang: String,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Force JSON output
    #[arg(long)]
    json: bool,

    /// Output format: pretty, json, ndjson, quiet
    #[arg(long)]
    output: Option<String>,

    /// Minimal output (cheapest price only)
    #[arg(short, long)]
    quiet: bool,

    /// Open the Google Flights URL in browser
    #[arg(long)]
    open: bool,
}

fn parse_cabin(s: &str) -> Result<Cabin, CrosswindError> {
    match s.to_lowercase().as_str() {
        "economy" | "e" => Ok(Cabin::Economy),
        "premium" | "premium-economy" | "pe" => Ok(Cabin::PremiumEconomy),
        "business" | "b" => Ok(Cabin::Business),
        "first" | "f" => Ok(Cabin::First),
        _ => Err(CrosswindError::Other(format!(
            "unknown cabin class '{s}', use economy, premium, business, or first"
        ))),
    }
}

fn validate_airport_code(code: &str) -> Result<String, CrosswindError> {
    let code = code.trim().to_uppercase();
    if code.len() != 3 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(CrosswindError::InvalidAirportCode(code));
    }
    Ok(code)
}

fn sort_flights(flights: &mut [Flight], sort: &str) {
    match sort {
        "duration" => flights.sort_by_key(|f| f.duration_minutes),
        "stops" => flights.sort_by_key(|f| (f.stops, f.price.max(0))),
        "departure" => flights.sort_by(|a, b| {
            let key = |f: &Flight| {
                f.segments
                    .first()
                    .map(|s| (s.depart_date.clone(), s.depart_time.clone()))
            };
            key(a).cmp(&key(b))
        }),
        // default: price ascending, unknown prices last
        _ => flights.sort_by(|a, b| {
            let pa = if a.price == 0 { i64::MAX } else { a.price };
            let pb = if b.price == 0 { i64::MAX } else { b.price };
            pa.cmp(&pb)
        }),
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    let format = match OutputFormat::detect(cli.json, cli.output.as_deref(), cli.quiet) {
        Ok(f) => f,
        Err(msg) => {
            eprintln!("error: {msg}");
            process::exit(2);
        }
    };

    if let Err(e) = run(&cli, format, start).await {
        let timing_ms = start.elapsed().as_millis() as u64;
        if format.is_machine() {
            output::print_error_json(&e, "search", timing_ms);
        } else {
            output::print_error_text(&e);
        }
        process::exit(e.exit_code());
    }
}

async fn run(cli: &Cli, format: OutputFormat, start: Instant) -> Result<(), CrosswindError> {
    let origin = validate_airport_code(&cli.origin)?;
    let destinations: Vec<String> = cli
        .destinations
        .split(',')
        .map(validate_airport_code)
        .collect::<Result<Vec<_>, _>>()?;

    if destinations.is_empty() {
        return Err(CrosswindError::Other("no destinations provided".into()));
    }

    let depart_date = parse_date(&cli.date)?;
    let return_date = cli.ret.as_deref().map(parse_date).transpose()?;
    let cabin = parse_cabin(&cli.cabin)?;

    if cli.adults == 0 || cli.adults > 9 {
        return Err(CrosswindError::InvalidPassengers(format!(
            "adults must be 1-9, got {}",
            cli.adults
        )));
    }

    let max_stops = if cli.nonstop { Some(0) } else { cli.max_stops };

    let params = QueryParams {
        origin,
        depart_date,
        return_date,
        cabin,
        adults: cli.adults,
        max_stops,
        currency: cli.currency.clone(),
        lang: cli.lang.clone(),
    };

    if cli.open {
        let url = crosswind::query::build_url(&params, &destinations[0]);
        open::that(&url)
            .map_err(|e| CrosswindError::Other(format!("failed to open browser: {e}")))?;
        if output::is_tty() {
            eprintln!("Opened in browser: {url}");
        }
        return Ok(());
    }

    let mut all_flights = Vec::new();
    let mut all_airlines = Vec::new();

    for dest in &destinations {
        match crosswind::search(&params, dest, cli.timeout).await {
            Ok(result) => {
                all_flights.extend(result.flights);
                all_airlines.extend(result.airlines);
            }
            Err(CrosswindError::NoResults) if destinations.len() > 1 => {
                eprintln!("{DIM}no flights for {dest}, skipping{RESET}");
            }
            Err(e) => return Err(e),
        }
    }

    if all_flights.is_empty() {
        return Err(CrosswindError::NoResults);
    }

    all_airlines.sort_by(|a, b| a.code.cmp(&b.code));
    all_airlines.dedup_by(|a, b| a.code == b.code);

    if let Some(max) = cli.max_duration {
        all_flights.retain(|f| f.duration_minutes <= max);
    }
    if let Some(max) = cli.max_price {
        all_flights.retain(|f| f.price > 0 && f.price <= max);
    }

    sort_flights(&mut all_flights, &cli.sort);

    if let Some(top) = cli.top {
        all_flights.truncate(top);
    }

    let result = SearchResult {
        flights: all_flights,
        airlines: all_airlines,
    };

    let timing_ms = start.elapsed().as_millis() as u64;

    match format {
        OutputFormat::Json => output::print_json(&result, "search", timing_ms),
        OutputFormat::Ndjson => output::print_ndjson(&result),
        OutputFormat::Quiet => output::print_quiet(&result, &cli.currency),
        OutputFormat::Pretty => {
            output::print_table(&result, &cli.currency);
            eprintln!(
                "\n{DIM}{} flights · {:.1}s{RESET}",
                result.flights.len(),
                timing_ms as f64 / 1000.0,
            );
        }
    }

    Ok(())
}
