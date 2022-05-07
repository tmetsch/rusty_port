extern crate console_error_panic_hook;

use std::panic;

use wasm_bindgen::{prelude, JsCast};

/// Chunk size for getting set of vessels.
const CHUNK_SIZE: usize = 200;

/// Url to use for the tiles.
const TILES_URL: &str =
    "https://cartodb-basemaps-{s}.global.ssl.fastly.net/light_all/{z}/{x}/{y}.png";

/// Endpoint to use to reach datapoint - temporary.
const DATAPORT_ENDPOINT: &str = "http://localhost:8000/vessels";

/// Options for the Polyline.
#[derive(serde::Serialize, serde::Deserialize)]
struct PolylineOptions {
    color: String,
    weight: usize,
}

/// Options for the icon.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct IconOptions {
    icon_url: String,
    icon_size: (usize, usize),
    icon_anchor: (i8, i8),
    popup_anchor: (i8, i8),
}

/// do a HTTP request to a specified endpoint.
async fn do_request<T: serde::Serialize>(
    url: &str,
    verb: &str,
    body: T,
) -> Result<prelude::JsValue, prelude::JsValue> {
    let mut opts = web_sys::RequestInit::new();
    opts.method(verb);
    opts.mode(web_sys::RequestMode::Cors);

    // if we do a POST we add whatever is in the body.
    if verb == "POST" {
        let json = serde_json::to_string(&body).unwrap();
        let json: prelude::JsValue = prelude::JsValue::from_serde(&json).unwrap();
        opts.body(Some(&json));
    }

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    // if we have a body it it json data.
    if verb == "POST" {
        request
            .headers()
            .set("Content-Type", "application/json")
            .unwrap();
    }

    let window = web_sys::window().unwrap();
    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<web_sys::Response>());
    let resp: web_sys::Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

    Ok(json)
}

/// Get a potential list of MMSIs from the query string.
fn get_query_vessels() -> Vec<i32> {
    let mut res: Vec<i32> = vec![];
    let window = web_sys::window().expect("the actual window.");
    let tmp = window.location().search().unwrap();
    let search_str = tmp.trim_start_matches('?');
    let queries: Vec<&str> = search_str.split('&').collect();
    for part in queries {
        if part.is_empty() {
            continue;
        }
        let kv_list: Vec<&str> = part.split('=').collect();
        if kv_list.first().copied().unwrap() != "mmsi" {
            continue;
        }
        let mssi_list: Vec<&str> = kv_list.last().copied().unwrap().split(',').collect();
        for mmsi in mssi_list {
            res.push(mmsi.parse::<i32>().unwrap());
        }
    }
    res
}

/// Get all vessels in the area.
async fn get_vessels() -> common::MMSIList {
    // FIXME: need to get this from were the browser loaded this form.
    let endpoint = DATAPORT_ENDPOINT;
    let json: prelude::JsValue = do_request(endpoint, "GET", None::<usize>)
        .await
        .expect("A map of vessels.");
    let vessels: common::MMSIList = json.into_serde().unwrap();
    vessels
}

/// add tiles to the leaflet map.
fn add_tiles(map: &leaflet::Map) {
    leaflet::TileLayer::new(TILES_URL, &prelude::JsValue::NULL).addTo(map);
}

/// Add an GPS trace.
fn add_traces(map: &leaflet::Map, traces: Vec<common::Vessel>) {
    for vessel in traces {
        let mut pos = vec![];
        for item in vessel.coordinates {
            pos.push(leaflet::LatLng::new(item.1, item.0));
        }
        let trace = leaflet::Polyline::new_with_options(
            pos.iter().map(prelude::JsValue::from).collect(),
            &prelude::JsValue::from_serde(&PolylineOptions {
                color: "#ff7900".into(),
                weight: 1,
            })
            .expect("simple line options."),
        );
        trace.addTo(map);

        // popup marker.
        let marker = leaflet::Marker::new(pos.last().unwrap());
        marker.setIcon(&leaflet::Icon::new(
            &prelude::JsValue::from_serde(&IconOptions {
                icon_url: "marker.png".into(),
                icon_size: (16, 16),
                icon_anchor: (8, 16),
                popup_anchor: (0, -8),
            })
            .expect("simple marker options."),
        ));
        let text: String = format!(
            "<strong><u>{}</u></strong> &raquo; <em><a href=\"?mmsi={}\" target=\"_blank\">show</a></em><br />\
            <strong>MMSI</strong>: <a href=\"https://www.marinetraffic.com/en/ais/details/ships/mmsi:{}\" target=\"_blank\">{}</a><br />\
            <strong>Speed</strong>: {}<br />\
            <strong>Heading</strong>:{}\
            <br /><strong>Type</strong>:{}\
            <br /><strong>Timestamp</strong>:{}, <br />\
            <strong>Status</strong>:{}, <br />\
            <strong>Destination</strong>:{}",
            vessel.name,
            vessel.mmsi,
            vessel.mmsi,
            vessel.mmsi,
            vessel.speeds.last().copied().unwrap(),
            vessel.headings.last().copied().unwrap(),
            vessel.ship_type, vessel.timestamps.last().cloned().unwrap(),
            vessel.statuses.last().cloned().unwrap(),
            vessel.destinations.last().cloned().unwrap(),
        );
        leaflet::Layer::bindPopup(
            &marker,
            &prelude::JsValue::from_str(&*text),
            &prelude::JsValue::NULL,
        );
        marker.addTo(map);
    }
}

/// Called by the javascript part.
#[prelude::wasm_bindgen(start)]
pub async fn main() -> Result<(), prelude::JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // initial map - focus on Rotterdam.
    let map = leaflet::Map::new("map", &prelude::JsValue::NULL);
    map.setView(&leaflet::LatLng::new(51.9496, 4.1453), 10.0);
    add_tiles(&map);

    // either there are ships given through the query part of the URI, otherwise we show all.
    let mut vessels = common::MMSIList {
        vessels: get_query_vessels(),
    };
    if vessels.vessels.is_empty() {
        vessels = get_vessels().await;
    }

    // FIXME: figure out async runtime for wasm.
    for vessel_chunk in vessels.vessels.chunks(CHUNK_SIZE) {
        let tmp = do_request(
            DATAPORT_ENDPOINT,
            "POST",
            common::MMSIList {
                vessels: Vec::from(vessel_chunk),
            },
        );
        let tracks: common::VesselList = tmp.await.unwrap().into_serde().unwrap();
        add_traces(&map, tracks.vessels);
    }

    // and ready to go.
    Ok(())
}

mod tests {
    // TODO: implement this!
}
