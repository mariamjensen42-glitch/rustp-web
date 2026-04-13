use std::collections::HashMap;

// 定义角色常量
pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_EDITOR: &str = "editor";
pub const ROLE_AUTHOR: &str = "author";
pub const ROLE_SUBSCRIBER: &str = "subscriber";

// 定义权限常量
pub const PERMISSION_READ_POSTS: &str = "read_posts";
pub const PERMISSION_CREATE_POSTS: &str = "create_posts";
pub const PERMISSION_EDIT_POSTS: &str = "edit_posts";
pub const PERMISSION_DELETE_POSTS: &str = "delete_posts";
pub const PERMISSION_MANAGE_CATEGORIES: &str = "manage_categories";
pub const PERMISSION_MANAGE_TAGS: &str = "manage_tags";
pub const PERMISSION_MANAGE_COMMENTS: &str = "manage_comments";
pub const PERMISSION_MANAGE_MEDIA: &str = "manage_media";
pub const PERMISSION_MANAGE_USERS: &str = "manage_users";
pub const PERMISSION_MANAGE_ROLES: &str = "manage_roles";

// 角色权限映射
pub fn get_role_permissions() -> HashMap<&'static str, Vec<&'static str>> {
    let mut permissions = HashMap::new();
    
    // 管理员拥有所有权限
    permissions.insert(ROLE_ADMIN, vec![
        PERMISSION_READ_POSTS,
        PERMISSION_CREATE_POSTS,
        PERMISSION_EDIT_POSTS,
        PERMISSION_DELETE_POSTS,
        PERMISSION_MANAGE_CATEGORIES,
        PERMISSION_MANAGE_TAGS,
        PERMISSION_MANAGE_COMMENTS,
        PERMISSION_MANAGE_MEDIA,
        PERMISSION_MANAGE_USERS,
        PERMISSION_MANAGE_ROLES,
    ]);
    
    // 编辑可以管理内容
    permissions.insert(ROLE_EDITOR, vec![
        PERMISSION_READ_POSTS,
        PERMISSION_CREATE_POSTS,
        PERMISSION_EDIT_POSTS,
        PERMISSION_DELETE_POSTS,
        PERMISSION_MANAGE_CATEGORIES,
        PERMISSION_MANAGE_TAGS,
        PERMISSION_MANAGE_COMMENTS,
        PERMISSION_MANAGE_MEDIA,
    ]);
    
    // 作者只能管理自己的内容
    permissions.insert(ROLE_AUTHOR, vec![
        PERMISSION_READ_POSTS,
        PERMISSION_CREATE_POSTS,
        PERMISSION_EDIT_POSTS,
        PERMISSION_DELETE_POSTS,
        PERMISSION_MANAGE_COMMENTS,
    ]);
    
    // 订阅者只能阅读
    permissions.insert(ROLE_SUBSCRIBER, vec![
        PERMISSION_READ_POSTS,
    ]);
    
    permissions
}

// 检查用户是否有特定权限
pub fn has_permission(role: &str, permission: &str) -> bool {
    let role_permissions = get_role_permissions();
    if let Some(permissions) = role_permissions.get(role) {
        permissions.contains(&permission)
    } else {
        false
    }
}

// 检查用户是否可以编辑特定帖子
pub fn can_edit_post(role: &str, user_id: i32, post_user_id: Option<i32>) -> bool {
    if role == ROLE_ADMIN || role == ROLE_EDITOR {
        true
    } else if role == ROLE_AUTHOR {
        post_user_id == Some(user_id)
    } else {
        false
    }
}

// 检查用户是否可以删除特定帖子
pub fn can_delete_post(role: &str, user_id: i32, post_user_id: Option<i32>) -> bool {
    can_edit_post(role, user_id, post_user_id)
}

// 检查用户是否可以编辑特定评论
pub fn can_edit_comment(role: &str, user_id: i32, comment_user_id: Option<i32>) -> bool {
    if role == ROLE_ADMIN || role == ROLE_EDITOR {
        true
    } else if role == ROLE_AUTHOR {
        comment_user_id == Some(user_id)
    } else {
        false
    }
}

// 检查用户是否可以删除特定评论
pub fn can_delete_comment(role: &str, user_id: i32, comment_user_id: Option<i32>) -> bool {
    can_edit_comment(role, user_id, comment_user_id)
}
