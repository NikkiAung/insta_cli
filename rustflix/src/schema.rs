// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        removed -> Bool,
    }
}

diesel::table! {
    videos (id) {
        id -> Int4,
        title -> Varchar,
        description -> Text,
        removed -> Bool,
    }
}

diesel::table! {
    views (id) {
        id -> Int4,
        video_id -> Int4,
        user_id -> Int4,
        watch_start -> Timestamp,
        duration -> Int4,
    }
}

diesel::joinable!(views -> users (user_id));
diesel::joinable!(views -> videos (video_id));

diesel::allow_tables_to_appear_in_same_query!(users, videos, views,);
