use actix_web::{App, get, HttpResponse, HttpServer, middleware, Responder, web};
use actix_web::error::JsonPayloadError;
use actix_web::http::header;
use serde::Serialize;
use data::SystemTimeTempData;
use uts_ws1::config::Config;
use status::SystemStatus;

mod status;
mod data;

struct AppState {
    app_name: String,
    config: Config,
}

#[get("/")]
async fn get_index(state: web::Data<AppState>) -> String {
    let app_name = &state.app_name; // <- get app_name
    format!("Hello {app_name}!") // <- response with app_name
}

#[get("/status")]
async fn get_status(state: web::Data<AppState>) -> impl Responder {
    let status = SystemStatus::read(&state.config);
    pretty_json(&status)
}

#[get("/data")]
async fn get_data(state: web::Data<AppState>) -> impl Responder {
    let status = SystemStatus::read(&state.config);
    let data = SystemTimeTempData::from(status);
    pretty_json(&data)
}

fn pretty_json<T>(result: &T) -> HttpResponse
    where T: Serialize {
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
    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(app_data.clone())
            .service(get_index)
            .service(
                web::scope("/api")
                    .service(get_status)
                    .service(get_data))
    })
        .bind(("0.0.0.0", 5001))?
        .run();
    eprintln!("uts-web: listening on port 5001...");
    server.await
}
