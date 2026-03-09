use std::process;
use std::time::Instant;

use clap::Parser;

use crosswind::date::parse_date;
use crosswind::error::CrosswindError;
use crosswind::output;
use crosswind::query::QueryParams;
use crosswind::query::proto::Cabin;

/// Google Flights from your terminal
#[derive(Parser)]
#[command(name = "crosswind", version, about)]
struct Cli {
    /// Origin airport code (e.g. BEG, JFK, LAX)
    origin: String,

    /// Destination airport code(s), comma-separated for multi-city (e.g. JFK or JFK,LHR)
    #[arg(name = "destination")]
    destinations: String,

    /// Departure date (apr1, 4/1, +7, tomorrow, 2026-04-01)
    date: String,

    /// Return date for round-trip (same formats as departure)
    #[arg(short, long)]
    ret: Option<String>,

    /// Cabin class: economy, premium, business, first
    #[arg(short, long, default_value = "economy")]
    cabin: String,

    /// Number of adult passengers
    #[arg(short = 'p', long, default_value = "1")]
    adults: u32,

    /// Maximum stops (0 = nonstop only)
    #[arg(long)]
    max_stops: Option<i32>,

    /// Currency code (USD, EUR, GBP, etc.)
    #[arg(long, default_value = "USD")]
    currency: String,

    /// Language code
    #[arg(long, default_value = "en")]
    lang: String,

    /// Request timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Force JSON output (even in TTY)
    #[arg(long)]
    json: bool,

    /// Open the Google Flights URL in browser instead of scraping
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
            "unknown cabin class '{s}' — use economy, premium, business, or first"
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        let use_json = !output::is_tty();
        if use_json {
            output::print_error_json(&e);
        } else {
            eprintln!("error: {e}");
            if let Some(hint) = e.hint() {
                eprintln!("hint: {hint}");
            }
        }
        process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> Result<(), CrosswindError> {
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

    let params = QueryParams {
        origin,
        destinations: destinations.clone(),
        depart_date,
        return_date,
        cabin,
        adults: cli.adults,
        max_stops: cli.max_stops,
        currency: cli.currency.clone(),
        lang: cli.lang.clone(),
    };

    // --open: just open the URL in browser
    if cli.open {
        let url = crosswind::query::build_url(&params, &destinations[0]);
        open::that(&url)
            .map_err(|e| CrosswindError::Other(format!("failed to open browser: {e}")))?;
        if output::is_tty() {
            eprintln!("Opened in browser: {url}");
        }
        return Ok(());
    }

    let use_json = cli.json || !output::is_tty();
    let start = Instant::now();

    // Search each destination
    let mut all_flights = Vec::new();
    let mut all_airlines = Vec::new();

    for dest in &destinations {
        let result = crosswind::search(&params, dest, cli.timeout).await?;
        all_flights.extend(result.flights);
        all_airlines.extend(result.airlines);
    }

    // Deduplicate airlines by code
    all_airlines.sort_by(|a, b| a.code.cmp(&b.code));
    all_airlines.dedup_by(|a, b| a.code == b.code);

    // Sort by price (0 = unknown, push to end)
    all_flights.sort_by(|a, b| {
        let pa = if a.price == 0 { i64::MAX } else { a.price };
        let pb = if b.price == 0 { i64::MAX } else { b.price };
        pa.cmp(&pb)
    });

    let result = crosswind::model::SearchResult {
        flights: all_flights,
        airlines: all_airlines,
    };

    let timing_ms = start.elapsed().as_millis() as u64;

    if use_json {
        output::print_json(&result, "search", timing_ms);
    } else {
        output::print_table(&result, &cli.currency);
        eprintln!(
            "\n{} flights found in {:.1}s",
            result.flights.len(),
            timing_ms as f64 / 1000.0,
        );
    }

    Ok(())
}
