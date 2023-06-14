use actix_cors::Cors;
use actix_web::{App, get, post, HttpResponse, HttpServer, middleware, Responder, web};
use actix_web::error::JsonPayloadError;
use actix_web::http::header;
use actix_web::web::Redirect;
use log::{error, info};
use serde::Serialize;
use data::SystemTimeTempData;
use uts_ws1::config::Config;
use status::SystemStatus;
use log_data::LogReader;
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
    format!("Hello {app_name}!") // <- response with app_name
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
    let boards = state.config.create_boards();
    let board = boards.iter().find(|b| b.bus.id == update.board_id);
    if let Some(board) = board {
        update.apply(board);
    } else {
        error!("Board ID not found or configured: {}", update.board_id);
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
    let data = &state.config.read_logs();
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
    let config = Config::read();
    let app_data = web::Data::new(AppState {
        app_name: String::from("Hestia control panel"),
        config: config.clone(),
    });
    let addr = ("0.0.0.0", config.http_port);
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("https://uts.dashboard.space")
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(app_data.clone())
            .service(get_index)
            .service(
                web::scope("/api")
                    .service(get_status)
                    .service(post_status)
                    .service(get_data)
                    .service(get_log_data))
    })
        .bind(addr)?
        .run();
    info!("uts-web listening on {:?}...", addr);
    server.await
}
