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

#[derive(QueryableByName)]
pub struct ProjectWithoutTest {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub id: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub program_id: i32,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub grade: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub grade_max: i32,
}

#[derive(Identifiable, Queryable, Selectable, Insertable)]
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
