use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use prost::Message;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/crosswind.rs"));
}

use proto::{Airport, Cabin, PassengerType, SearchRequest, Segment, TripType};

pub struct QueryParams {
    pub origin: String,
    pub destinations: Vec<String>,
    pub depart_date: String,
    pub return_date: Option<String>,
    pub cabin: Cabin,
    pub adults: u32,
    pub max_stops: Option<i32>,
    pub currency: String,
    pub lang: String,
}

pub fn build_url(params: &QueryParams, destination: &str) -> String {
    let tfs = encode_tfs(params, destination);
    format!(
        "https://www.google.com/travel/flights/search?tfs={}&hl={}&curr={}&tfu=EgQIABABIgA",
        tfs, params.lang, params.currency,
    )
}

fn encode_tfs(params: &QueryParams, destination: &str) -> String {
    let mut segments = vec![Segment {
        date: params.depart_date.clone(),
        origin: Some(Airport {
            code: params.origin.clone(),
        }),
        destination: Some(Airport {
            code: destination.to_string(),
        }),
        max_stops: params.max_stops,
    }];

    let trip = if let Some(ref ret_date) = params.return_date {
        segments.push(Segment {
            date: ret_date.clone(),
            origin: Some(Airport {
                code: destination.to_string(),
            }),
            destination: Some(Airport {
                code: params.origin.clone(),
            }),
            max_stops: params.max_stops,
        });
        TripType::RoundTrip
    } else {
        TripType::OneWay
    };

    let passengers: Vec<i32> = (0..params.adults)
        .map(|_| PassengerType::Adult as i32)
        .collect();

    let request = SearchRequest {
        segments,
        passengers,
        cabin: params.cabin as i32,
        trip: trip as i32,
    };

    let mut buf = Vec::new();
    request.encode(&mut buf).expect("protobuf encode failed");
    URL_SAFE_NO_PAD.encode(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_way_encodes() {
        let params = QueryParams {
            origin: "BEG".into(),
            destinations: vec!["JFK".into()],
            depart_date: "2026-04-01".into(),
            return_date: None,
            cabin: Cabin::Economy,
            adults: 1,
            max_stops: None,
            currency: "USD".into(),
            lang: "en".into(),
        };
        let url = build_url(&params, "JFK");
        assert!(url.contains("tfs="));
        assert!(url.contains("curr=USD"));
        assert!(url.contains("hl=en"));

        // Decode and verify the protobuf roundtrips
        let tfs = url.split("tfs=").nth(1).unwrap().split('&').next().unwrap();
        let bytes = URL_SAFE_NO_PAD.decode(tfs).unwrap();
        let req = SearchRequest::decode(bytes.as_slice()).unwrap();
        assert_eq!(req.segments.len(), 1);
        assert_eq!(req.segments[0].date, "2026-04-01");
        assert_eq!(req.segments[0].origin.as_ref().unwrap().code, "BEG");
        assert_eq!(req.segments[0].destination.as_ref().unwrap().code, "JFK");
        assert_eq!(req.trip, TripType::OneWay as i32);
        assert_eq!(req.cabin, Cabin::Economy as i32);
        assert_eq!(req.passengers, vec![PassengerType::Adult as i32]);
    }

    #[test]
    fn round_trip_encodes() {
        let params = QueryParams {
            origin: "BEG".into(),
            destinations: vec!["JFK".into()],
            depart_date: "2026-04-01".into(),
            return_date: Some("2026-04-16".into()),
            cabin: Cabin::Business,
            adults: 2,
            max_stops: None,
            currency: "EUR".into(),
            lang: "en".into(),
        };
        let url = build_url(&params, "JFK");
        let tfs = url.split("tfs=").nth(1).unwrap().split('&').next().unwrap();
        let bytes = URL_SAFE_NO_PAD.decode(tfs).unwrap();
        let req = SearchRequest::decode(bytes.as_slice()).unwrap();
        assert_eq!(req.segments.len(), 2);
        assert_eq!(req.segments[0].date, "2026-04-01");
        assert_eq!(req.segments[0].origin.as_ref().unwrap().code, "BEG");
        assert_eq!(req.segments[1].date, "2026-04-16");
        assert_eq!(req.segments[1].origin.as_ref().unwrap().code, "JFK");
        assert_eq!(req.segments[1].destination.as_ref().unwrap().code, "BEG");
        assert_eq!(req.trip, TripType::RoundTrip as i32);
        assert_eq!(req.cabin, Cabin::Business as i32);
        assert_eq!(req.passengers.len(), 2);
    }

    #[test]
    fn nonstop_filter() {
        let params = QueryParams {
            origin: "LAX".into(),
            destinations: vec!["JFK".into()],
            depart_date: "2026-04-01".into(),
            return_date: None,
            cabin: Cabin::Economy,
            adults: 1,
            max_stops: Some(0),
            currency: "USD".into(),
            lang: "en".into(),
        };
        let url = build_url(&params, "JFK");
        let tfs = url.split("tfs=").nth(1).unwrap().split('&').next().unwrap();
        let bytes = URL_SAFE_NO_PAD.decode(tfs).unwrap();
        let req = SearchRequest::decode(bytes.as_slice()).unwrap();
        assert_eq!(req.segments[0].max_stops, Some(0));
    }
}
