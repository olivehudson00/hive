/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

pub mod output;
pub mod models;
pub mod schema;

use std::error::Error;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

use axum::{
    extract::{Multipart, Path as AxumPath, State},
    response::Html,
    routing::{get, post},
    Router,
    Server,
};
use deadpool_diesel::sqlite::{Manager, Pool, Runtime};
use diesel::prelude::*;
use flate2::read::GzDecoder;
use tar::Archive;
use tempfile::TempDir;
use tokio::process::Command;

use crate::models::*;
use crate::schema::*;
use crate::output;

fn date(timestamp: u64) -> String {
    return format!("{}", timestamp);
}

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
        .route("/:pr", post(post_submit))
        .with_state(pool)
        .fallback_service(ServeDir::new("static"));

    Server::bind(&"127.0.0.1:5000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
