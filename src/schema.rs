// @generated automatically by Diesel CLI.

diesel::table! {
    enrolments (id) {
        id -> Integer,
        user_id -> Integer,
        program_id -> Integer,
    }
}

diesel::table! {
    programs (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    projects (id) {
        id -> Integer,
        program_id -> Integer,
        name -> Text,
        test -> Binary,
        grade -> Integer,
    }
}

diesel::table! {
    submissions (id) {
        id -> Integer,
        user_id -> Integer,
        project_id -> Integer,
        time -> Timestamp,
        results -> Nullable<Text>,
        grade -> Nullable<Integer>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(enrolments -> users (user_id));
diesel::joinable!(projects -> programs (program_id));
diesel::joinable!(submissions -> projects (project_id));
diesel::joinable!(submissions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(enrolments, programs, projects, submissions, users,);
