use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub flights: Vec<Flight>,
    pub airlines: Vec<Airline>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Flight {
    pub airlines: Vec<String>,
    pub segments: Vec<FlightSegment>,
    pub price: i64,
    pub stops: i32,
    pub duration_minutes: i32,
    pub is_best: bool,
    pub carbon_grams: Option<i64>,
    pub typical_carbon_grams: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FlightSegment {
    pub from_code: String,
    pub from_name: String,
    pub to_code: String,
    pub to_name: String,
    pub depart_date: String,
    pub depart_time: String,
    pub arrive_date: String,
    pub arrive_time: String,
    pub duration_minutes: i32,
    pub aircraft: Option<String>,
    pub flight_number: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Airline {
    pub code: String,
    pub name: String,
}
