diesel::table! {
    categories (id) {
        id -> Integer,
        name -> Text,
        slug -> Nullable<Text>,
        description -> Nullable<Text>,
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
    media (id) {
        id -> Integer,
        filename -> Text,
        filepath -> Text,
        mimetype -> Text,
        size -> Bigint,
        uploaded_by -> Nullable<Integer>,
        created_at -> Timestamp,
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
        slug -> Nullable<Text>,
        content -> Text,
        excerpt -> Nullable<Text>,
        author -> Text,
        status -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        published_at -> Nullable<Timestamp>,
        deleted_at -> Nullable<Timestamp>,
        category_id -> Nullable<Integer>,
        user_id -> Nullable<Integer>,
        summary -> Nullable<Text>,
        cover_image -> Nullable<Text>,
        is_published -> Nullable<Bool>,
        is_top -> Nullable<Bool>,
        allow_comments -> Nullable<Bool>,
        view_count -> Nullable<Integer>,
        scheduled_at -> Nullable<Timestamp>,
        is_scheduled -> Nullable<Bool>,
        draft_saved_at -> Nullable<Timestamp>,
        auto_save_draft -> Nullable<Bool>,
    }
}

diesel::table! {
    post_versions (id) {
        id -> Integer,
        post_id -> Integer,
        version_number -> Integer,
        title -> Text,
        content -> Text,
        excerpt -> Nullable<Text>,
        summary -> Nullable<Text>,
        cover_image -> Nullable<Text>,
        created_by -> Nullable<Integer>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    post_analytics (id) {
        id -> Integer,
        post_id -> Integer,
        visit_date -> Date,
        visit_count -> Integer,
        unique_visitors -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
        slug -> Nullable<Text>,
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

diesel::table! {
    user_read_history (id) {
        id -> Integer,
        user_id -> Integer,
        post_id -> Integer,
        read_at -> Timestamp,
        read_duration -> Nullable<Integer>,
    }
}

diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(user_read_history -> users (user_id));
diesel::joinable!(user_read_history -> posts (post_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(media -> users (uploaded_by));
diesel::joinable!(post_tags -> posts (post_id));
diesel::joinable!(post_tags -> tags (tag_id));
diesel::joinable!(posts -> categories (category_id));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(post_versions -> posts (post_id));
diesel::joinable!(post_versions -> users (created_by));
diesel::joinable!(post_analytics -> posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(categories, comments, media, post_tags, posts, tags, users, post_versions, post_analytics, user_read_history);
