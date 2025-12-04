/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

use diesel::prelude::*;

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::programs)]
pub struct Program {
    pub id: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Program))]
#[diesel(table_name = crate::schema::enrolments)]
pub struct Enrolment {
    pub id: i32,
    pub user_id: i32,
    pub program_id: i32,
}

#[derive(Identifiable, Queryable, Selectable, Associations)]
#[diesel(belongs_to(Program))]
#[diesel(table_name = crate::schema::projects)]
pub struct Project {
    pub id: i32,
    pub program_id: i32,
    pub name: String,
    pub grade: i32,
}

#[derive(Identifiable, Queryable, Selectable, Associations, Insertable)]
#[diesel(belongs_to(Project))]
#[diesel(table_name = crate::schema::submissions)]
pub struct Submission {
    pub id: i32,
    pub user_id: i32,
    pub project_id: i32,
    pub time: chrono::NaiveDateTime,
    pub results: Option<String>,
    pub grade: Option<i32>,
}

#[derive(Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::projects)]
pub struct ProjectWithTest {
    pub id: i32,
    pub program_id: i32,
    pub name: String,
    pub test: Vec<u8>,
    pub grade: i32,
}
