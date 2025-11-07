// @generated automatically by Diesel CLI.

diesel::table! {
    enrolments (id) {
        id -> Integer,
        user -> Integer,
        program -> Integer,
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
        program -> Integer,
        name -> Text,
        test -> Binary,
    }
}

diesel::table! {
    submissions (id) {
        id -> Integer,
        user -> Integer,
        project -> Integer,
        time -> BigInt,
        results -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::joinable!(enrolments -> users (user));
diesel::joinable!(projects -> programs (program));
diesel::joinable!(submissions -> projects (project));
diesel::joinable!(submissions -> users (user));

diesel::allow_tables_to_appear_in_same_query!(enrolments, programs, projects, submissions, users,);
