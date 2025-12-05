/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write as IoWrite;
use std::path::PathBuf;

use axum::{
    Extension,
    extract::{Multipart, Path as AxumPath, State},
    http::StatusCode,
    response::{Html, Redirect},
};
use deadpool_diesel::sqlite::Pool;
use diesel::{
    insert_into,
    prelude::*,
    sql_query,
};
use flate2::read::GzDecoder;
use tar::Archive;
use tempfile::TempDir;
use tokio::process::Command;

use crate::models::*;
use crate::schema;

fn e404<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::NOT_FOUND, err.to_string())
}

fn e500<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub async fn get_program(
    State(pool): State<Pool>,
    Extension(user_id): Extension<i32>,
) -> Result<Html<String>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(e500)?;

    let programs = conn.interact(move |conn| {
        schema::programs::table
            .inner_join(schema::enrolments::table.on(schema::enrolments::program_id.eq(schema::programs::id)))
            .filter(schema::enrolments::user_id.eq(user_id))
            .select(Program::as_select())
            .load(conn)
    }).await.map_err(e500)?.map_err(e500)?;

    // todo sort by then use chunk_by to group with parents to avoid O(n^2) loops
    let mut programs_iter = programs.iter();
    let mut list = format!("{}", programs_iter.next().unwrap().id);
    for program in programs_iter {
        write!(list, ", {}", program.id);
    }
    let query = format!(
        concat!(
            "SELECT p.id, p.program_id, p.name, p.grade, s.grade\n",
            "FROM projects AS p\n",
            "LEFT JOIN (\n",
            "    SELECT s.project_id, s.user_id, MAX(s.grade)\n",
            "    FROM submission AS s\n",
            "    GROUP BY s.project_id, s.user_id\n",
            ") AS s\n",
            "ON p.id = s.project_id AND s.user_id = {}\n",
            "WHERE p.program_id IN ({})"
        ),
        user_id,
        list
    );

    let projects: Vec<ProjectWithoutTest> = conn.interact(move |conn| {
        sql_query(query).get_results(conn)
    }).await.map_err(e500)?.map_err(e500)?;

    let mut string = String::new();
    write!(string, include_str!("head.html"), "Programs");
    for program in programs {
        write!(string, "<fieldset>\n");
        write!(string, "<legend>{}</legend>\n", program.name);

        for project in &projects {
            if project.program_id != program.id {
                continue
            }

            write!(string, "<div>\n");
            write!(string, "<a href='/{}'>{}</a>\n", project.id, project.name);
            write!(string, "<span></span>\n");
            write!(string, "<span>{}/{}</span>\n", project.grade, project.grade_max);
            write!(string, "</div>\n");
        }

        write!(string, "</fieldset>\n");
    }
    write!(string, include_str!("foot.html"));
    Ok(Html(string))
}

pub async fn get_project(
    State(pool): State<Pool>,
    Extension(user_id): Extension<i32>,
    AxumPath(project_id): AxumPath<i32>,
) -> Result<Html<String>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(e500)?;

    let (project_name, project_grade) = conn.interact(move |conn| {
        schema::projects::table
            .filter(schema::projects::id.eq(project_id))
            .select((
                schema::projects::name,
                schema::projects::grade,
            ))
            .first::<(String, i32)>(conn)
    }).await.map_err(e500)?.map_err(e500)?;
    let subs = conn.interact(move |conn| {
        schema::submissions::table
            .filter(schema::submissions::user_id.eq(user_id))
            .filter(schema::submissions::project_id.eq(project_id))
            .select(Submission::as_select())
            .load(conn)
    }).await.map_err(e500)?.map_err(e500)?;

    let mut string = String::new();
    write!(string, include_str!("head.html"), project_name);
    for sub in subs {
        write!(string, "<fieldset>\n");
        if let (Some(results), Some(grade)) = (sub.results, sub.grade) {
            write!(string, "<legend>{}——{}/{}</legend>\n", project_name, grade, project_grade);
            write!(string, "{}", results);
        } else {
            write!(string, "<legend>{}——0/{}</legend>\n", project_name, project_grade);
            write!(string, "<p>Awaiting Testing...</p>");
        }
        write!(string, "</fieldset>\n");
    }
    write!(string, include_str!("foot.html"));
    Ok(Html(string))
}

pub async fn post_project(
    State(pool): State<Pool>,
    Extension(user_id): Extension<i32>,
    AxumPath(project_id): AxumPath<i32>,
    mut form: Multipart,
) -> Result<Redirect, (StatusCode, String)> {
    let conn = pool.get().await.map_err(e500)?;

    let project = conn.interact(move |conn| {
        schema::projects::table
            .filter(schema::projects::id.eq(project_id))
            .select(ProjectWithTest::as_select())
            .first(conn)
    }).await.map_err(e500)?.map_err(e500)?;

    let dir = TempDir::new().map_err(e500)?;
    let mut path = PathBuf::new();
    path.push(dir.path());
    let tar = GzDecoder::new(&*project.test);
    let mut archive = Archive::new(tar);
    archive.unpack(&path);

    path.push("user");
    let mut file = File::create(&path).map_err(e500)?;
    let field = form
        .next_field()
        .await
        .map_err(e500)?
        .ok_or((StatusCode::BAD_REQUEST, "no file provided".to_string()))?;
    file.write_all(&field.bytes().await.map_err(e500)?).map_err(e500)?;
    path.pop();

    let mut sub = conn.interact(move |conn| {
        insert_into(schema::submissions::table)
            .values((
                schema::submissions::user_id.eq(user_id),
                schema::submissions::project_id.eq(project_id),
            ))
            .get_result::<Submission>(conn)
    }).await.map_err(e500)?.map_err(e500)?;

    tokio::spawn(async move {
        path.push("run.sh");
        let output = Command::new(&path)
            .output()
            .await;

        sub.results = Some(String::new());
        sub.grade = Some(0);
        if let Ok(output) = output {
            if let Some((output, grade)) = str::from_utf8(&output.stdout)
                .unwrap_or("")
                .rsplit_once('\n')
            {
                sub.results = Some(output.to_string());
                sub.grade = Some(grade.parse::<i32>().unwrap_or(0));
            }
        }

        conn.interact(move |conn| {
            insert_into(schema::submissions::table)
                .values(&sub)
                .execute(conn)
        }).await;
    });

    Ok(Redirect::to(&format!("/{}", project_id)))
}
