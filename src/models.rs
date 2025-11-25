/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    id: i32,
    name: String,
    nonce: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::programs)]
pub struct Program {
    id: i32,
    name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::enrolments)]
#[diesel(belongs_to(User, foreign_key = user))]
#[diesel(belongs_to(Program, foreign_key = program))]
pub struct Enrolment {
    id: i32,
    user: i32,
    program: i32,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(belongs_to(Program, foreign_key = program))]
pub struct Project {
    id: i32,
    program: i32,
    name: String,
    test: Vec<u8>,
    grade: i32,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(belongs_to(Program, foreign_key = program))]
struct ProjectSelect {
    id: i32,
    program: i32,
    name: String,
    grade: i32,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::submissions)]
#[diesel(belongs_to(User, foreign_key = user))]
#[diesel(belongs_to(Project, foreign_key = project))]
pub struct Submission {
    id: i32,
    user: i32,
    project: i32,
    time: u64,
    results: Option<String>,
    grade: Option<i32>,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(table_name = crate::schema::submissions)]
#[diesel(belongs_to(User, foreign_key = user))]
#[diesel(belongs_to(Project, foreign_key = project))]
pub struct SubmissionSelect {
    id: i32,
    user: i32,
    project: i32,
    grade: Option<i32>,
}
