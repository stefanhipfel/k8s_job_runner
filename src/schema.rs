// @generated automatically by Diesel CLI.

diesel::table! {
    jobs (id) {
        id -> Integer,
        name -> Text,
        version -> Nullable<Text>,
        docker_image -> Text,
        docker_image_tag -> Text,
    }
}

diesel::table! {
    maintenances (id) {
        id -> Nullable<Integer>,
        uuid -> Text,
        name -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        failed_attempts -> Integer,
        status -> Text,
        scheduled_for -> Nullable<Timestamp>,
        downtime_window_start -> Nullable<Timestamp>,
        downtime_window_end -> Nullable<Timestamp>,
        job_id -> Integer,
    }
}

diesel::joinable!(maintenances -> jobs (job_id));

diesel::allow_tables_to_appear_in_same_query!(
    jobs,
    maintenances,
);
