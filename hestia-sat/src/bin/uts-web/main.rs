use std::fmt;
use std::iter::zip;
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, middleware};
use actix_web::error::JsonPayloadError;
use actix_web::http::header;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeMap;
use uts_ws1::board::{Board, BoardData, CsvDataProvider};
use uts_ws1::config::Config;
use uts_ws1::heater::HeaterMode;
use uts_ws1::reading::SensorReading;
use uts_ws1::{board, ReadResult};
use uts_ws1::sensors::Sensor;

struct AppState {
    app_name: String,
    config: Config,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SensorInfo {
    id: String,
    label: String,
    pos_x: f32,
    pos_y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BoardStatus {
    sensor_info: LinkedHashMap<String, SensorInfo>,

    #[serde(serialize_with = "serialize_sensor_values")]
    sensor_values: LinkedHashMap<String, Option<f32>>,

    heater_mode: Option<HeaterMode>,

    #[serde(serialize_with = "serialize_temp")]
    target_temp: Option<f32>,
    target_sensor: Option<String>,
    heater_duty: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SystemStatus(LinkedHashMap<String, Option<BoardStatus>>);

fn serialize_temp<S>(value: &Option<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    match value {
        Some(value) => serializer.serialize_str(format!("{:0.2}", value).as_str()),
        None => serializer.serialize_none(),
    }
}

fn serialize_sensor_values<S>(values: &LinkedHashMap<String, Option<f32>>, serializer: S)
    -> Result<S::Ok, S::Error>
    where S: Serializer {
    let mut map = serializer.serialize_map(Some(values.len()))?;
    for (key, value) in values {
        match value {
            Some(value) => map.serialize_entry(key, format!("{:0.2}", value).as_str())?,
            None => map.serialize_entry(key, &None::<String>)?,
        }
    }
    map.end()
}

impl BoardStatus {
    fn from_data(board: &Board, data: BoardData) -> Self {
        let mut sensor_values = LinkedHashMap::with_capacity(board.sensors.len());
        for (sensor, value) in zip(board::ALL_SENSORS, data.sensors) {
            sensor_values.insert(String::from(sensor.id), from_reading(value));
        }
        let heater_mode = from_reading(data.heater_mode);
        let heater_duty = from_reading(data.pwm_freq);
        let target_temp = from_reading(data.target_temp);
        let target_sensor = from_reading(data.target_sensor).map(|s| s.to_string());
        BoardStatus {
            sensor_info: to_sensor_info(board::ALL_SENSORS),
            sensor_values,
            heater_mode,
            target_temp,
            target_sensor,
            heater_duty,
        }
    }
}

fn to_sensor_info<const N: usize>(sensors: &[Sensor; N]) -> LinkedHashMap<String, SensorInfo> {
    let mut sensor_info = LinkedHashMap::with_capacity(sensors.len());
    for sensor in sensors {
        sensor_info.insert(sensor.id.to_string(), SensorInfo {
            id: sensor.id.to_string(),
            label: sensor.label.to_string(),
            pos_x: sensor.pos_x,
            pos_y: sensor.pos_y,
        });
    }
    sensor_info
}

fn from_reading<T>(reading: ReadResult<SensorReading<T>>) -> Option<T>
    where T: fmt::Display {
    reading.ok().map(|v| v.display_value)
}

impl SystemStatus {
    fn new() -> Self {
        SystemStatus(LinkedHashMap::new())
    }

    fn add(&mut self, board: &Board) {
        let data = board.read_data();
        let data = data.map(|d| BoardStatus::from_data(board, d));
        self.0.insert(board.bus.id.to_string(), data);
    }
}

#[get("/")]
async fn index(state: web::Data<AppState>) -> String {
    let app_name = &state.app_name; // <- get app_name
    format!("Hello {app_name}!") // <- response with app_name
}

#[get("/status")]
async fn status(state: web::Data<AppState>) -> impl Responder {
    let mut result = SystemStatus::new();
    for board in state.config.create_boards() {
        result.add(&board);
    }
    match serde_json::to_string_pretty(&result) {
        Ok(body) => {
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, mime::APPLICATION_JSON))
                .body(body)
        }
        Err(err) => HttpResponse::from_error(JsonPayloadError::Serialize(err))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState {
        app_name: String::from("Hestia control panel"),
        config: Config::read(),
    });
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(app_data.clone())
            .service(index)
            .service(
                web::scope("/api")
                    .service(status))
    })
        .bind(("0.0.0.0", 5001))?
        .run()
        .await
}
