use std::collections;

/// Radius of the earth - let's hope this stays constant :-)
const RADIUS_EARTH: f64 = 6378137.0;

/// Represents a port.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Port {
    pub vessels: collections::HashMap<i32, Vessel>,
}

/// a GPS coordinate.
#[derive(serde::Serialize, serde::Deserialize, Default, Copy, Clone)]
pub struct Coordinate(pub f64, pub f64);

/// Represents a vessel within a port.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Vessel {
    pub mmsi: i32,
    pub name: String,
    pub ship_type: String,
    pub coordinates: Vec<Coordinate>,
    pub timestamps: Vec<String>,
    pub speeds: Vec<f64>,
    pub headings: Vec<f64>,
    pub statuses: Vec<String>,
    pub destinations: Vec<String>,
}

/// List of Vessels.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VesselList {
    pub vessels: Vec<Vessel>,
}

/// List of vessel identifiers - using MMSIs.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MMSIList {
    pub vessels: Vec<i32>,
}

/// Defines the input to the path simplification function.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SimplifyIn {
    pub vessel: Vessel,
    pub radius: f64,
}

/// Defines the output to the path simplification function.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SimplifyOut {
    pub path: Vec<Coordinate>,
}

/// Calculates distance between two coordinates - based on: <https://en.wikipedia.org/wiki/Haversine_formula>.
pub fn distance(src_lat: f64, src_long: f64, trg_lat: f64, trg_long: f64) -> f64 {
    let src_lat_rad = src_lat.to_radians();
    let trg_lat_rad = trg_lat.to_radians();

    let lat_delta_rad = (src_lat - trg_lat).to_radians();
    let long_delta_rad = (src_long - trg_long).to_radians();

    let tmp: f64 = (lat_delta_rad / 2.0).sin().powi(2)
        + src_lat_rad.cos() * trg_lat_rad.cos() * (long_delta_rad / 2.0).sin().powi(2);
    let central_angle = 2.0 * tmp.sqrt().asin();

    RADIUS_EARTH * central_angle
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for success.

    #[test]
    fn test_distance_for_success() {
        distance(50.0, 2.0, 40.0, 2.0);
    }

    // Tests for failure.

    // n/A

    // Tests for sanity.

    #[test]
    fn test_distance_for_sanity() {
        let res: f64 = distance(52.3676, 4.9041, 51.9244, 4.4777);
        assert_eq!(
            res.floor(),
            57293.0,
            "Distance between Amsterdam and Rotterdam should be ~57 km."
        );

        let res: f64 = distance(52.3676, 4.9041, 41.8781, -82.6298);
        assert_eq!(
            res.floor(),
            6317918.0,
            "Distance between Amsterdam and Chicago should be ~6318 km."
        );

        let res: f64 = distance(-34.83333, -58.5166646, 49.0083899664, 2.53844117956);
        assert_eq!(
            res.floor(),
            11111974.0,
            "Distance between Paris and Buenos Aires should be ~11112 km."
        );
    }
}
