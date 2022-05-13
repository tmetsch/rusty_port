#[macro_use]
extern crate rocket;

use rocket::serde::json;

#[post("/simplify", format = "application/json", data = "<data>")]
fn simplify(data: json::Json<common::SimplifyIn>) -> Option<json::Json<common::SimplifyOut>> {
    if data.vessel.coordinates.len() <= 1 {
        return None;
    }
    let mut optimized_path: Vec<common::Coordinate> = vec![];
    let tmp = data.vessel.coordinates.first().unwrap(); // I can unwrap as I know there is a first elem.
    optimized_path.push(*tmp);

    for n in 1..data.vessel.coordinates.len() {
        let src_coord = &optimized_path.last().unwrap();
        let trg_coord = &data.vessel.coordinates[n];
        let distance = common::distance(src_coord.0, src_coord.1, trg_coord.0, trg_coord.1);
        if distance > data.radius {
            optimized_path.push(*trg_coord)
        }
    }

    if optimized_path.len() < 2 {
        None
    } else {
        Some(json::Json(common::SimplifyOut {
            path: optimized_path,
        }))
    }
}

#[catch(default)]
fn error() -> &'static str {
    "Whoops doopsie."
}

#[launch]
fn rocket() -> _ {
    let figment = rocket::Config::figment()
        .merge(("port", 8765))
        .merge(("address", "0.0.0.0"));
    rocket::custom(figment)
        .mount("/", routes![simplify])
        .register("/", catchers![error])
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn test_rocket_for_success() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get("/").dispatch();
        assert_eq!(
            response.status(),
            Status::from_code(404).expect("should be 404")
        );
        // assert_eq!(response.into_string(), Some("Hello, world!".into()));
    }

    #[test]
    fn test_simplify_for_success() {
        let item = common::SimplifyIn {
            vessel: common::Vessel {
                mmsi: 123,
                name: "Boaty McBoatface".to_string(),
                ship_type: "dummy".to_string(),
                coordinates: vec![
                    common::Coordinate(51.453254021051386, 0.7516262537890542),
                    common::Coordinate(51.457077325466926, 0.762528238401656),
                    common::Coordinate(51.461044608406944, 0.7766441233094546),
                    common::Coordinate(51.46357330397952, 0.789980784315918),
                ],
                timestamps: vec![],
                speeds: vec![],
                headings: vec![],
                statuses: vec![],
                destinations: vec![],
            },
            radius: 100.0,
        };
        simplify(json::Json(item));
    }

    #[test]
    fn test_simplify_for_failure() {
        // only one coordinate.
        let item = common::SimplifyIn {
            vessel: common::Vessel {
                mmsi: 123,
                name: "Boaty McBoatface".to_string(),
                ship_type: "dummy".to_string(),
                coordinates: vec![common::Coordinate(51.453254021051386, 0.7516262537890542)],
                timestamps: vec![],
                speeds: vec![],
                headings: vec![],
                statuses: vec![],
                destinations: vec![],
            },
            radius: 100.0,
        };
        let res = simplify(json::Json(item));
        assert_eq!(res.is_none(), true, "This shouldn't happen.");

        // two equal coordinates.
        let item = common::SimplifyIn {
            vessel: common::Vessel {
                mmsi: 123,
                name: "Boaty McBoatface".to_string(),
                ship_type: "dummy".to_string(),
                coordinates: vec![
                    common::Coordinate(51.453254021051386, 0.7516262537890542),
                    common::Coordinate(51.453254021051386, 0.7516262537890542),
                ],
                timestamps: vec![],
                speeds: vec![],
                headings: vec![],
                statuses: vec![],
                destinations: vec![],
            },
            radius: 100.0,
        };
        let res = simplify(json::Json(item));
        assert_eq!(res.is_none(), true, "This shouldn't happen.")
    }

    #[test]
    fn test_simplify_for_sanity() {
        let item = common::SimplifyIn {
            vessel: common::Vessel {
                mmsi: 123,
                name: "Boaty McBoatface".to_string(),
                ship_type: "dummy".to_string(),
                coordinates: vec![
                    common::Coordinate(51.453254021051386, 0.7516262537890542),
                    common::Coordinate(51.457077325466926, 0.762528238401656),
                    common::Coordinate(51.461044608406944, 0.7766441233094546),
                    common::Coordinate(51.461044608405944, 0.7776441233094546),
                    common::Coordinate(51.46357330397952, 0.789980784315918),
                ],
                timestamps: vec![],
                speeds: vec![],
                headings: vec![],
                statuses: vec![],
                destinations: vec![],
            },
            radius: 100.0,
        };
        let res = simplify(json::Json(item));
        if let Some(v) = res {
            assert_eq!(v.path.len(), 4); // shortened by one step.
        }
    }
}
