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

async fn home(
    State(pool): State<Pool>,
) -> Result<Html<String>, String> {
    let conn = pool.get().await.map_err(|e| format!("aaaa {}", e))?;

    let projects: Vec<Project> = conn.interact(|conn| {
        schema::projects::table
            .select(Project::as_select())
            .load(conn)
    }).await.map_err(|e| format!("{}", e))?.map_err(|e| format!("{}", e))?;

    let mut string = String::new();
    for project in projects {
        write!(string, "<form action='/{}' method='post' enctype='multipart/form-data'>\n", project.id);
        write!(string, "<label for='{}'>Select a file: </label>\n", project.id);
        write!(string, "<input type='file' id='{}' name='file'>\n", project.id);
        write!(string, "<input type='submit' value='Submit to {}'>", project.name);
        write!(string, "</form>\n");
    }
    return Ok(axum::response::Html(string));
}

async fn accept(
    State(pool): State<Pool>,
    AxumPath(projectid): AxumPath<i32>,
    mut form: Multipart,
) -> Result<Html<String>, String> {
    let conn = pool.get().await.map_err(|e| format!("{}", e))?;

    let project = conn.interact(move |conn| {
        schema::projects::table
            .filter(projects::id.eq(projectid))
            .select(Project::as_select())
            .load(conn)
    }).await.map_err(|e| format!("{}", e))?.map_err(|e| format!("{}", e))?;

    if project.len() == 0 {
        return Err("no project found".to_string());
    }

    let dir = TempDir::new().map_err(|e| format!("{}", e))?;

    let mut path = PathBuf::new();
    path.push(dir.path());

    /* TODO */
    std::env::set_current_dir(&path);

    let tar = GzDecoder::new(&*project[0].test);
    let mut archive = Archive::new(tar);
    archive.unpack(&path);

    path.push("user");
    let field = form.next_field().await.map_err(|e| format!("ugh {}", e))?.unwrap();
    let mut file = File::create(&path).map_err(|e| format!("{}", e))?;
    file.write_all(&field.bytes().await.map_err(|e| format!("ughhhh {}", e))?).map_err(|e| format!("{}", e))?;
    path.pop();

    path.push("compile.sh");
    let output = Command::new(&path)
        .output()
        .await.map_err(|e| format!("ajfsdkljakdf {}", e))?;
    path.pop();

    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        return Ok(format!("Failed to compile!\n{}", String::from_utf8_lossy(&output.stdout)));
    }

    path.push("run.sh");
    let output = Command::new(&path)
        .output()
        .await.map_err(|e| format!("{}", e))?;

    return Ok(axum::response::Html(output::fmt(String::from_utf8_lossy(&output.stdout))));
}

#[tokio::main]
async fn main() {
    dotenv::dotenv();

    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL should be set");
    let manager = Manager::new(url, Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    let app = Router::new()
        .route("/", get(get_programs))
        .route("/:pr", get(get_project))
        .route("/:pr", post(post_submit))
        .with_state(pool);

    Server::bind(&"127.0.0.1:5000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
