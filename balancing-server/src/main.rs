use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

use std::env;

// INFO: struct Hello and handler hello are placeholder
#[derive(Serialize)]
struct Hello {
    hello: String,
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(Hello {
        hello: "world".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Check environment variable
    let port = match env::var("PORT") {
        Ok(val) => val.parse().unwrap(),
        Err(_) => 3500,
    };
    log::info!("Run on port :{}", port);

    // Run server
    HttpServer::new(|| App::new().wrap(Logger::default()).service(hello))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
