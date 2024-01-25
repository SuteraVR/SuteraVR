use axum::{
    extract::Request,
    middleware::Next,
    response::{Json, Response},
    routing::get,
    Router,
};
use tower::ServiceBuilder;

use serde::Serialize;

use http::StatusCode;
use std::env;

// INFO: struct Hello and handler hello are placeholder
#[derive(Serialize)]
struct Hello {
    hello: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

async fn hello() -> Json<Hello> {
    Json(Hello {
        hello: "world".to_string(),
    })
}

async fn my_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    match request.headers().get("SuteraVR-SchemaVersion") {
        Some(version) if version == VERSION => {}
        _ => {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("SuteraVR-SchemaVersion", VERSION.parse().unwrap());

    Ok(response)
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Check environment variable
    let port = match env::var("PORT") {
        Ok(val) => val.parse().unwrap(),
        Err(_) => 3500,
    };
    log::info!("Run on port :{}", port);

    let app = Router::new()
        .route("/hello", get(hello))
        .layer(ServiceBuilder::new().layer(axum::middleware::from_fn(my_middleware)));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    // Run server
    axum::serve(listener, app).await.unwrap();
}
