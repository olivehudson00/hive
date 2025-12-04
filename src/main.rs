/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

pub mod models;
pub mod routes;
pub mod schema;

use axum::{
    routing::{get, post},
    Router,
    serve
};
use deadpool_diesel::sqlite::{Manager, Pool, Runtime};
use tower_http::services::ServeDir;

use crate::routes::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv();

    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL should be set");
    let manager = Manager::new(url, Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    let app = Router::new()
        .route("/", get(get_program))
        .route("/:pr", get(get_project))
        .route("/:pr", post(post_project))
        .fallback_service(ServeDir::new("static"))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000")
        .await
        .unwrap();
    serve(listener, app.into_make_service())
        .await
        .unwrap();
}
