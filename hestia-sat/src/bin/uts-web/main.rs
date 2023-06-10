use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, middleware};
use actix_web::error::JsonPayloadError;
use actix_web::http::header;
use serde::Serialize;
use uts_ws1::config::Config;
use crate::status::SystemStatus;

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
    let mut result = SystemStatus::new();
    for board in state.config.create_boards() {
        result.add(&board);
    }
    pretty_json(&result)
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
                    .service(get_status))
    })
        .bind(("0.0.0.0", 5001))?
        .run();
    eprintln!("uts-web: listening on port 5001...");
    server.await
}
