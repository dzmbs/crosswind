use scraper::{Html, Selector};
use serde_json::Value;

use crate::error::CrosswindError;
use crate::model::{Airline, Flight, FlightSegment, SearchResult};

pub fn parse(html: &str) -> Result<SearchResult, CrosswindError> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"script.ds\:1"#).expect("valid selector");

    let script = document
        .select(&selector)
        .next()
        .ok_or(CrosswindError::ScriptTagNotFound)?;

    let js = script.text().collect::<String>();

    let data_str = js
        .split("data:")
        .nth(1)
        .ok_or_else(|| CrosswindError::ParseError("no 'data:' marker in script".into()))?;

    let data_str = data_str
        .rsplit_once(',')
        .map(|(before, _)| before)
        .unwrap_or(data_str);

    let payload: Value = serde_json::from_str(data_str)
        .map_err(|e| CrosswindError::ParseError(format!("JSON parse failed: {e}")))?;

    let mut flights = Vec::new();

    // Parse "Best flights" from payload[2][0]
    if let Some(best_list) = payload
        .get(2)
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_array())
    {
        for item in best_list {
            if let Some(flight) = parse_flight(item, true) {
                flights.push(flight);
            }
        }
    }

    // Parse "Other results" from payload[3][0]
    if let Some(other_list) = payload
        .get(3)
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_array())
    {
        for item in other_list {
            if let Some(flight) = parse_flight(item, false) {
                flights.push(flight);
            }
        }
    }

    if flights.is_empty() {
        return Err(CrosswindError::NoResults);
    }

    // Parse airline metadata from payload[7][1]
    let airlines = parse_airlines(&payload);

    Ok(SearchResult { flights, airlines })
}

fn parse_flight(item: &Value, is_best: bool) -> Option<Flight> {
    let fd = item.get(0)?;
    let airlines = fd
        .get(1)?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<Vec<_>>();

    let segments_arr = fd.get(2)?.as_array()?;
    let mut segments = Vec::new();

    for seg in segments_arr {
        if let Some(s) = parse_segment(seg) {
            segments.push(s);
        }
    }

    if segments.is_empty() {
        return None;
    }

    let price = item
        .get(1)
        .and_then(|v| v.get(0))
        .and_then(|v| v.get(1))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let stops = (segments.len() as i32) - 1;

    let duration_minutes = fd.get(9).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let carbon_grams = fd.get(22).and_then(|v| v.get(7)).and_then(|v| v.as_i64());

    let typical_carbon_grams = fd.get(22).and_then(|v| v.get(8)).and_then(|v| v.as_i64());

    Some(Flight {
        airlines,
        segments,
        price,
        stops,
        duration_minutes,
        is_best,
        carbon_grams,
        typical_carbon_grams,
    })
}

fn parse_segment(seg: &Value) -> Option<FlightSegment> {
    let from_code = seg.get(3)?.as_str()?.to_string();
    let from_name = seg
        .get(4)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let to_name = seg
        .get(5)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let to_code = seg.get(6)?.as_str()?.to_string();

    let depart_time = format_time(seg.get(8));
    let arrive_time = format_time(seg.get(10));
    let depart_date = format_date(seg.get(20));
    let arrive_date = format_date(seg.get(21));

    let duration_minutes = seg.get(11).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let aircraft = seg.get(17).and_then(|v| v.as_str()).map(String::from);

    let flight_number = parse_flight_number(seg.get(22));

    Some(FlightSegment {
        from_code,
        from_name,
        to_code,
        to_name,
        depart_date,
        depart_time,
        arrive_date,
        arrive_time,
        duration_minutes,
        aircraft,
        flight_number,
    })
}

fn format_time(val: Option<&Value>) -> String {
    let arr = match val.and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return String::new(),
    };
    let hour = arr.first().and_then(|v| v.as_i64()).unwrap_or(0);
    let minute = arr.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
    format!("{hour:02}:{minute:02}")
}

fn format_date(val: Option<&Value>) -> String {
    let arr = match val.and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return String::new(),
    };
    let year = arr.first().and_then(|v| v.as_i64()).unwrap_or(0);
    let month = arr.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
    let day = arr.get(2).and_then(|v| v.as_i64()).unwrap_or(0);
    format!("{year:04}-{month:02}-{day:02}")
}

fn parse_flight_number(val: Option<&Value>) -> Option<String> {
    let arr = val?.as_array()?;
    let airline = arr.first()?.as_str()?;
    let number = arr.get(1)?.as_str()?;
    Some(format!("{airline} {number}"))
}

fn parse_airlines(payload: &Value) -> Vec<Airline> {
    let mut airlines = Vec::new();

    let airlines_data = payload
        .get(7)
        .and_then(|v| v.get(1))
        .and_then(|v| v.get(1))
        .and_then(|v| v.as_array());

    if let Some(list) = airlines_data {
        for item in list {
            if let Some(arr) = item.as_array() {
                let code = arr
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = arr
                    .get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !code.is_empty() {
                    airlines.push(Airline { code, name });
                }
            }
        }
    }

    airlines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_works() {
        let val = serde_json::json!([14, 30]);
        assert_eq!(format_time(Some(&val)), "14:30");
    }

    #[test]
    fn format_time_hour_only() {
        let val = serde_json::json!([9]);
        assert_eq!(format_time(Some(&val)), "09:00");
    }

    #[test]
    fn format_time_none() {
        assert_eq!(format_time(None), "");
    }

    #[test]
    fn format_date_works() {
        let val = serde_json::json!([2026, 4, 1]);
        assert_eq!(format_date(Some(&val)), "2026-04-01");
    }

    #[test]
    fn parse_flight_number_works() {
        let val = serde_json::json!(["LO", "572", null, "LOT"]);
        assert_eq!(parse_flight_number(Some(&val)), Some("LO 572".into()));
    }

    #[test]
    fn parse_flight_number_missing() {
        assert_eq!(parse_flight_number(None), None);
    }
}
