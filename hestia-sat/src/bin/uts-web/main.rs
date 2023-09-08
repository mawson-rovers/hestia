use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{App, get, post, HttpResponse, HttpServer, middleware, Responder, web};
use actix_web::error::JsonPayloadError;
use actix_web::http::header;
use actix_web::middleware::Condition;
use actix_web::web::Redirect;
use log::{error, info};
use serde::Serialize;
use data::SystemTimeTempData;
use uts_ws1::payload::{Config, Payload};
use status::SystemStatus;
use crate::status::BoardStatusUpdate;

mod data;
mod log_data;
mod status;

struct AppState {
    app_name: String,
    config: Config,
}

#[get("/")]
async fn get_index(state: web::Data<AppState>) -> String {
    let app_name = &state.app_name; // <- get app_name
    format!("Welcome to {app_name}! Try /api/status or /api/data for more info.")
}

#[get("/status")]
async fn get_status(state: web::Data<AppState>) -> impl Responder {
    let status = SystemStatus::read(&state.config);
    pretty_json(&status)
}

#[post("/status")]
async fn post_status(state: web::Data<AppState>, update: web::Json<BoardStatusUpdate>)
    -> impl Responder {
    let update = update.into_inner();
    let payload = Payload::from_config(&state.config);
    let board = payload.iter().find(|b| b.bus.id == update.board as u8);
    if let Some(board) = board {
        update.apply(board);
    } else {
        error!("Board ID not found or configured: {}", update.board);
    }
    Redirect::to("/api/status").see_other()
}

#[get("/data")]
async fn get_data(state: web::Data<AppState>) -> impl Responder {
    let status = SystemStatus::read(&state.config);
    let data = SystemTimeTempData::from(status);
    pretty_json(&data)
}

#[get("/log_data")]
async fn get_log_data(state: web::Data<AppState>) -> impl Responder {
    let data = log_data::read_logs(&state.config);
    pretty_json(&data)
}

#[get("/log_files")]
async fn get_log_files(state: web::Data<AppState>) -> impl Responder {
    let log_files = log_data::list_logs(&state.config);
    pretty_json(&log_files)
}

#[get("/log/{name}")]
async fn download_log(state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let log_file = log_data::get_log_file(&state.config, &name);
    NamedFile::open(log_file)
}

fn pretty_json<T>(result: &T) -> HttpResponse
    where T: Serialize {
    match serde_json::to_string_pretty(result) {
        Ok(body) => {
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, "application/json; charset=utf-8"))
                .body(body)
        }
        Err(err) => HttpResponse::from_error(JsonPayloadError::Serialize(err))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::read();
    let app_data = web::Data::new(AppState {
        app_name: String::from("Hestia API"),
        config: config.clone(),
    });
    let addr = ("0.0.0.0", config.http_port);
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(middleware::Compress::default())
            .wrap(Condition::new(config.cors_enable, cors))
            .app_data(app_data.clone())
            .service(get_index)
            .service(
                web::scope("/api")
                    .service(get_status)
                    .service(post_status)
                    .service(get_data)
                    .service(get_log_data)
                    .service(get_log_files)
                    .service(download_log)
            )
    })
        .bind(addr)?
        .run();
    info!("uts-web listening on {:?}...", addr);
    server.await
}
