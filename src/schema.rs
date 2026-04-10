// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        content -> Text,
        author -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
