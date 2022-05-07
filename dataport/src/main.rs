#[macro_use]
extern crate rocket;

use std::{fs, io};

use rocket::serde::json;

/// Retrieve a list of vessels.
#[get("/vessels")]
fn vessels(data: &rocket::State<common::Port>) -> json::Json<common::MMSIList> {
    let mmsi = data.vessels.keys().copied().collect();
    json::Json(common::MMSIList { vessels: mmsi })
}

/// Retrieve details about a particular set vessels.
#[post("/vessels", format = "application/json", data = "<mmsis>")]
fn vessels_status(
    mmsis: json::Json<common::MMSIList>,
    data: &rocket::State<common::Port>,
) -> json::Json<Option<common::VesselList>> {
    let mut status = vec![];
    for item in &mmsis.vessels {
        if data.vessels.contains_key(item) {
            status.push(data.vessels.get(item).cloned().unwrap());
        }
    }
    if !status.is_empty() {
        json::Json(Option::from(common::VesselList { vessels: status }))
    } else {
        json::Json(None)
    }
}

/// Returns an emtpy index page.
#[get("/")]
fn index() -> &'static str {
    "Nothing to see here."
}

/// Returns example port data as defined in a JSON file.
fn get_port_data() -> common::Port {
    let file = fs::File::open("data.json").expect("Expected data.json.");
    let buffered_reader = io::BufReader::new(file);
    json::serde_json::from_reader(buffered_reader).expect("Could not parse json.")
}

/// Launches the rocket engine.
#[launch]
fn rocket() -> _ {
    // Get some port data - will replace with MongoDB or similar soon.
    let data: common::Port = get_port_data();

    // Configure rocket engines
    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 8000));
    rocket::custom(figment)
        .mount("/", routes![index, vessels, vessels_status])
        .manage(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http;
    use rocket::local::blocking;
    use std::collections;

    // Tests for success.

    #[test]
    fn test_index_for_success() {
        index();
    }

    #[test]
    fn test_vessels_for_success() {
        let rocket = rocket::build().manage(common::Port {
            vessels: Default::default(),
        });
        let data = rocket::State::get(&rocket).expect("Port state.`");
        vessels(data);
    }

    #[test]
    fn test_vessels_status_for_success() {
        let rocket = rocket::build().manage(common::Port {
            vessels: Default::default(),
        });
        let data = rocket::State::get(&rocket).expect("Port state.`");
        let mmsis = json::Json(common::MMSIList { vessels: vec![] });
        vessels_status(mmsis, data);
    }

    // Tests for failure.

    // n/a.

    // Tests for sanity.

    #[test]
    fn test_index_for_sanity() {
        let res = index();
        assert_eq!(res, "Nothing to see here.");
    }

    #[test]
    fn test_vessels_for_sanity() {
        let vessel = common::Vessel {
            mmsi: 123,
            name: "boaty mcboatface".to_string(),
            ship_type: "special".to_string(),
            coordinates: vec![],
            timestamps: vec![],
            speeds: vec![],
            headings: vec![],
            statuses: vec![],
            destinations: vec![],
        };
        let rocket = rocket::build().manage(common::Port {
            vessels: collections::HashMap::from([(123, vessel)]),
        });
        let data = rocket::State::get(&rocket).expect("Port state.`");
        let res = vessels(data);
        assert_eq!(res.vessels.len(), 1);
        assert_eq!(res.vessels[0], 123);
    }

    #[test]
    fn test_vessels_status_for_sanity() {
        // empty list --> empty result.
        let rocket = rocket::build().manage(common::Port {
            vessels: Default::default(),
        });
        let data = rocket::State::get(&rocket).expect("Port state.`");
        let mmsi = json::Json(common::MMSIList { vessels: vec![] });
        let res = vessels_status(mmsi, data);
        assert_eq!(res.is_none(), true);

        // non existing mmsi.
        let mmsi = json::Json(common::MMSIList { vessels: vec![456] });
        let res = vessels_status(mmsi, data);
        assert_eq!(res.is_none(), true);

        // success.
        let vessel = common::Vessel {
            mmsi: 456,
            name: "foo".to_string(),
            ship_type: "bar".to_string(),
            coordinates: vec![],
            timestamps: vec![],
            speeds: vec![],
            headings: vec![],
            statuses: vec![],
            destinations: vec![],
        };
        let rocket = rocket::build().manage(common::Port {
            vessels: collections::HashMap::from([(456, vessel)]),
        });
        let data = rocket::State::get(&rocket).expect("Port state.`");
        let mmsi = json::Json(common::MMSIList { vessels: vec![456] });
        let res = vessels_status(mmsi, data);
        assert_eq!(res.0.unwrap().vessels.len(), 1);
    }

    #[test]
    fn test_rocket_for_sanity() {
        let client = blocking::Client::tracked(rocket()).expect("a valid test client.");
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), http::Status::Ok);
        assert_eq!(response.into_string(), Some("Nothing to see here.".into()));
    }
}
