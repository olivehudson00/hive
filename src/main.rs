/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

pub mod models;
pub mod schema;

use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Extension, Multipart, Path},
    Router,
    Server,
};
use deadpool_diesel::sqlite::{Manager, Pool, Runtime};
use tokio::process::Command;

fn date(timestamp: u64) -> String {
    return format!("{}", timestamp);
}

async fn home(
    State(pool): State<Pool>,
    Path(userid): Path<i32>
) -> String {
    let conn = pool.get().await.map_err(internal)?;

    let user = conn.interact(|conn| {
        schema::users::table
            .filter(schema::users::id.eq(userid))
            .select(User::as_select())
            .get_result(conn)
    }).await.map_err(internal)?.map_err(internal)?;
    let programs = conn.interact(|conn| {
        Enrolment::belonging_to(&user)
            .inner_join(programs::table)
            .select(Program::as_select())
            .load(conn)
    }).await.map_err(internal)?.map_err(internal)?;
    let projects = conn.interact(|conn| {
        Project::belonging_to(&programs)
            .select(Project::as_select())
            .load(conn)
    }).await.map_err(internal)?.map_err(internal)?;
    let programs = projects
        .grouped_by(&programs)
        .into_iter()
        .zip(programs)
        .map(|(projects, program)| (program, projects))
        .collect::<Vec<(Program, Vec<Project>)>>;

    let mut string = String::new();
    for (program, projects) in programs {
        write!(string, "<details open=''>\n<summary>{}</summary>\n", program.name);
        for project in projects {
            write!(string, "<a href='/{}/{}'>{}</a>", userid, project.id, project.name);
        }
        write!(string, "</details>\n");
    }

    return string;
}

async fn submit(
    State(pool): State<Pool>,
    Path(userid): Path<i32>,
    Path(projectid): Path<i32>
) -> String {
    let conn = pool.get().await.map_err(internal)?;

    let project = conn.interact(|conn| {
        schema::projects::table
            .filter(schema::projects::id.eq(projectid))
            .select(Project::as_select())
            .get_result(conn)
    }).await.map_err(internal)?.map_err(internal)?;
    let submissions = conn.interact(|conn| {
        Submission::belonging_to(&project)
            .filter(schema::submissions::.user.eq(userid))
            .select(Submission::as_select())
            .order(schema::submissions::time.desc())
            .load(conn)
    }).await.map_err(internal)?.map_err(internal)?;

    let mut string = String::new();
    write!(string, "<form action='/{}/{}' method='post' enctype='multipart/form-data'>\n\
        <input type='file' name='file'>\n<input type='submit' value='Submit'>\n</form>\n",
        userid, project.id);

    if submissions.len() == 0 {
        write!(string, "<p>No submissions yet.</p>\n");
        return string;
    }

    for submission in submissions.iter().rev() {
        write!("<details open=''>\n");
        write!("<summary>{}</summary>\n", date(submission.time));
        write!("<p>{}</p>\n", match submission.results {
            Some(results) => results,
            None => "Awaiting results...",
        });
        write!("</details>\n")
    }
    return string;
}

async fn accept(
    State(pool): State<Pool>,
    Path(userid): Path<i32>,
    Path(projectid): Path<i32>,
    form: Multipart
) -> Result<(), Box<dyn Error>> {
    

    /* get only first file, ignore any others */
    match form.next_field().await {
        Ok(Some(field)) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let name = format!("{}-{}", userid, now);
            let data = field.bytes().await.unwrap();

            let mut file = File::create(name)?;
            file.write_all(data)?;

            let mut command = Command::new(test)
                .arg(name)
                .output()
                .await
                .map_err(internal)?;

            
        }
        Err(error) => todo!(),
        _ => (),
    }
}

#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL should be set");
    let manager = Manager::new(url, Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    let path = Path::new("/var/www/hive");
    std::fs::create_dir_all(path).expect("unable to create /var/www/hive");
    std::env::set_current_dir(path).unwrap("unable to move to /var/www/hive");

    let app = Router::new()
        .route("/:id", get(home))
        .route("/:id/:pr", get(submit))
        .route("/:id/:pr", post(accept))
        .with_state(pool);

    Server::bind("127.0.0.1:5000".parse().unwrap())
        .serve(app.into_make_serivce())
        .await
        .unwrap();
}
