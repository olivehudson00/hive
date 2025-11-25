/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

fn e500<E>(err: E) -> impl IntoResponse
where
    E: std::error::Error,
{
    Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

fn transform_results(results: String) -> String {
    let lines: Vec<&str> = results.split('\n').collect();
    let string = String::new();
    for i in (0..lines.len()) {
        
    }

}

async fn get_program(
    State(pool): State<Pool>,
    Extension(userid): Extension<i32>,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(e500);

    let programs = conn.interact(|conn| {
        schema::enrolments::table
            .filter(schema::enrolments::user.eq(userid))
            .inner_join(programs::table)
            .select(Program::as_select())
            .load(conn)
    }).await.map_err(e500)?.map_err(e500)?;
    let projects = conn.interact(|conn| {
        ProjectSelect::belonging_to(&programs)
            .select(ProjectSelect::as_select())
            .load(conn)
            .grouped_by(&programs)
    }).await.map_err(e500)?.map_err(e500)?;
    let submissions = conn.interact(|conn| {
        SubmissionSelect::belonging_to(&projects)
            .filter(schema::submission::user.eq(userid))
            .select(SubmissionSelect::as_select())
            .load(conn)
            .grouped_by(&projects)
    }).await.map_err(e500)?.map_err(e500)?;

    let projects = projects
        .into_iter()
        .zip(submissions)
        .collect::<Vec<(ProjectSelect, Vec<SubmissionSelect>)>>;
    let programs = programs
        .into_iter()
        .zip(projects)
        .collect::<Vec<(Program, Vec<(ProjectSelect, Vec<SubmissionSelect>)>)>>;

    let string = String::new();
    for (program, projects) in programs {
        write!(string, "<fieldset>\n");
        write!(string, "<legend>{}</legend>\n", program.name);

        for (project, submissions) in projects {
            write!(string, "<div>\n");
            write!(string, "<a href='/{}'>{}</a>\n", project.id, project.name);
            write!(string, "<span></span>\n");
            write!(string, "<span>{}/{}</span>\n", submissions.iter().max_by_key(|e| if e.grade.is_some() { e.grade } else { 0 }).unwrap_or(0), project.grade);
            write!(string, "</div>\n");
        }

        write!(string, "</fieldset>\n");
    }

    Ok(string)
}

async fn get_project(
    State(pool): State<Pool>,
    Extension(userid): Extension<i32>,
    AxumPath(projectid): AxumPath<i32>,
) -> impl IntoResponse {
    let conn = pool.get().await.map_err(e500);

    let project = conn.interact(|conn| {
        schema::projects::table
            .filter(schema::projects::id.eq(projectid))
            .select(ProjectSelect::as_select())
            .load(conn)
    }).await.map_err(e500)?.map_err(e500)?;
    let submissions = conn.interact(|conn| {
        Submission::belonging_to(&project)
            .filter(schema::submission::user.eq(userid))
            .select(Submission::as_select())
            .load(conn)
    }).await.map_err(e500)?.map_err(e500)?;

    let string = String::new();
    for submission in submissions {
        write!(string, "<fieldset>\n");

        if (Some(results), Some(grade)) = (submission.results, submission.grade) {
            write!(string, "<legend>{}——{}/{}</legend>\n", project.name, grade, project.grade);
            
            
        } else {
            write!(string, "<legend>{}——0/{}</legend>\n", project.name, project.grade);
            write!(string, "<p>Awaiting Testing...</p>");
        }

        write!(string, "</fieldset>\n");
    }
}

async fn post_project(
    State(pool): State<Pool>,
    Extension(ctx): Extension<Auth>,
    AxumPath(projectid): AxumPath<i32>,
    mut form: Multipart,
) -> impl IntoResponse {
    
}
