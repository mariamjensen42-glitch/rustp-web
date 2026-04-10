// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Integer,
        name -> Text,
        parent_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    comments (id) {
        id -> Integer,
        post_id -> Integer,
        user_id -> Nullable<Integer>,
        parent_id -> Nullable<Integer>,
        content -> Text,
        author_name -> Nullable<Text>,
        author_email -> Nullable<Text>,
        author_website -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    post_tags (post_id, tag_id) {
        post_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        content -> Text,
        author -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        category_id -> Nullable<Integer>,
        user_id -> Nullable<Integer>,
        summary -> Nullable<Text>,
        cover_image -> Nullable<Text>,
        is_published -> Nullable<Bool>,
        is_top -> Nullable<Bool>,
        allow_comments -> Nullable<Bool>,
        view_count -> Nullable<Integer>,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        role -> Text,
        avatar -> Nullable<Text>,
        bio -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(post_tags -> posts (post_id));
diesel::joinable!(post_tags -> tags (tag_id));
diesel::joinable!(posts -> categories (category_id));
diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(categories, comments, post_tags, posts, tags, users,);
