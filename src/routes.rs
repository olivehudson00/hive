/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

fn e500<E>(err: E) -> impl IntoResponse
where
    E: std::error::Error,
{
    Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

pub async fn get_program(
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
    write!(string, include_str!("head.html"), "Programs");
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
    write!(string, include_str!("foot.html"));
    Ok(string)
}

pub async fn get_project(
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
    write!(string, include_str!("head.html"), project.name);
    for submission in submissions {
        write!(string, "<fieldset>\n");
        if (Some(results), Some(grade)) = (submission.results, submission.grade) {
            write!(string, "<legend>{}——{}/{}</legend>\n", project.name, grade, project.grade);
            write!(string, "{}", results);
        } else {
            write!(string, "<legend>{}——0/{}</legend>\n", project.name, project.grade);
            write!(string, "<p>Awaiting Testing...</p>");
        }
        write!(string, "</fieldset>\n");
    }
    write!(string, include_str!("foot.html"));
    Ok(string)
}

pub async fn post_project(
    State(pool): State<Pool>,
    Extension(userid): Extension<i32>,
    AxumPath(projectid): AxumPath<i32>,
    mut form: Multipart,
) -> impl IntoResponse {
    let conn = pool.get.await.map_err(e500)?;

    let project = conn.interact(|conn| {
        schema::projects::table
            .filter(schema::projects::id.eq(projectid))
            .select(Project::as_select())
            .load(conn)
    }).await.map_err(e500)?.map_err(e500)?;
    if project.len() == 0 {
        return Err(StatusCode::NOT_FOUND, "no such project");
    }
    let project = project[0];

    let dir = TempDir::new().map_err(e500)?;
    let mut path = PathBuf::new();
    path.push(dir.path());
    let tar = GzDecoder::new(&project.test);
    let mut archive = Archive::new(tar);
    archive.unpack(&path);

    path.push("user");
    let mut file = File::create(&path).map_err(e500)?;
    let field = form
        .next_field()
        .await
        .map_err(e500)?
        .ok_or((StatusCode::BAD_REQUEST, "no file provided"))?;
    file.write_all(&field.bytes().await.map_err(e500)?).map_err(e500)?;
    path.pop();

    let mut sub = conn.interact(|conn| {
        insert_into(schema::submissions::table)
            .values((
                schema::submissions::user.eq(userid),
                schema::submissions::project.eq(projectid),
            ))
            .get_result::<Submission>(conn)

    }).await.map_err(e500)?.map_err(e500)?;

    tokio::spawn(async move {
        path.push("run.sh");
        let output = Command::new(&path)
            .output()
            .await
            .map_err(e500)?;

        let (output, grade) = output.rsplit_once('\n');
        sub.results = Some(output);
        sub.grade = Some(i32::try_into(grade).unwrap_or(0));
        conn.interact(|conn| {
            insert_into(schema::submissions::table)
                .values(&sub)
                .execute(conn)
        }).await.map_err(e500)?.map_err(e500)?;
    });

    Ok(Redirect::to(format!("/{}", projectid)))
}
