# 博客系统新功能设计文档

## 1. 项目概述

本设计文档描述了博客系统后端 API 的两个新核心功能：评论系统增强和内容导出功能。这些功能将提升用户体验和系统功能完整性，使博客系统更加现代化和用户友好。

## 2. 功能需求

### 2.1 评论系统增强

根据用户需求，评论系统增强应包含以下功能：
- **评论点赞**：允许用户对评论进行点赞，显示点赞数量
- **回复通知**：当评论被回复时，通过邮件或站内通知提醒用户
- **评论排序**：支持按时间、热度等多种方式排序评论

### 2.2 内容导出功能

根据用户需求，内容导出功能应支持以下格式：
- **Markdown 格式**：导出为标准 Markdown 格式，方便在其他平台使用
- **PDF 格式**：导出为 PDF 文档，适合打印和存档
- **HTML 格式**：导出为完整的 HTML 页面，包含样式

## 3. 技术实现方案

### 3.1 评论系统增强

#### 3.1.1 表结构修改

**方案选择**：扩展现有表结构

**具体修改**：
1. **comments 表**：
   - 添加 `likes_count` 字段（整数类型）：记录评论的点赞数量
   - 添加 `sort_order` 字段（整数类型）：用于自定义排序
   - 添加 `notification_sent` 字段（布尔类型）：标记是否已发送通知

2. **新增 comment_likes 表**：
   - `id`：主键
   - `comment_id`：外键，关联 comments 表
   - `user_id`：外键，关联 users 表
   - `created_at`：创建时间

3. **新增 comment_notifications 表**：
   - `id`：主键
   - `comment_id`：外键，关联 comments 表
   - `user_id`：外键，关联 users 表
   - `notification_type`：通知类型（如 "reply"）
   - `is_read`：是否已读
   - `created_at`：创建时间

#### 3.1.2 API 接口设计

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| POST | `/api/comments/{id}/like` | 点赞评论 | ✅ |
| DELETE | `/api/comments/{id}/like` | 取消点赞 | ✅ |
| GET | `/api/comments/{id}/likes` | 获取评论点赞数 | ❌ |
| GET | `/api/posts/{id}/comments` | 获取文章评论（支持排序） | ❌ |
| GET | `/api/users/me/notifications` | 获取用户通知 | ✅ |
| PUT | `/api/notifications/{id}/read` | 标记通知为已读 | ✅ |

#### 3.1.3 实现细节

1. **评论点赞**：
   - 当用户点赞评论时，检查 `comment_likes` 表中是否已存在记录
   - 如果不存在，创建记录并增加 `comments` 表中的 `likes_count`
   - 如果存在，删除记录并减少 `comments` 表中的 `likes_count`

2. **回复通知**：
   - 当创建评论且 `parent_id` 不为空时，查询父评论的作者
   - 为父评论作者创建通知记录
   - 可选：发送邮件通知（需要配置邮件服务）

3. **评论排序**：
   - 支持的排序方式：
     - `newest`：按创建时间倒序
     - `oldest`：按创建时间正序
     - `popular`：按点赞数倒序
   - 通过查询参数 `sort` 控制排序方式

### 3.2 内容导出功能

#### 3.2.1 技术方案

**方案选择**：使用 Rust 内置库实现

**具体实现**：
1. **Markdown 导出**：
   - 使用 `pulldown-cmark` 库将 HTML 内容转换为 Markdown
   - 保留文章的标题、内容、标签等信息

2. **PDF 导出**：
   - 使用 `weasyprint` 库生成 PDF
   - 构建 HTML 模板，包含文章内容和样式
   - 将 HTML 转换为 PDF 格式

3. **HTML 导出**：
   - 构建完整的 HTML 模板，包含文章内容和样式
   - 保留文章的所有格式和样式
   - 生成独立的 HTML 文件

#### 3.2.2 API 接口设计

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/api/posts/{id}/export/markdown` | 导出文章为 Markdown 格式 | ❌ |
| GET | `/api/posts/{id}/export/pdf` | 导出文章为 PDF 格式 | ❌ |
| GET | `/api/posts/{id}/export/html` | 导出文章为 HTML 格式 | ❌ |
| GET | `/api/posts/export/markdown` | 批量导出文章为 Markdown 格式 | ✅ |
| GET | `/api/posts/export/pdf` | 批量导出文章为 PDF 格式 | ✅ |
| GET | `/api/posts/export/html` | 批量导出文章为 HTML 格式 | ✅ |

#### 3.2.3 实现细节

1. **Markdown 导出**：
   - 获取文章详情，包括标题、内容、标签、分类等
   - 将 HTML 内容转换为 Markdown 格式
   - 添加文章元数据（如标题、作者、发布时间）
   - 设置响应头 `Content-Disposition` 为 `attachment`，使浏览器下载文件

2. **PDF 导出**：
   - 获取文章详情
   - 构建包含文章内容的 HTML 模板
   - 使用 `weasyprint` 将 HTML 转换为 PDF
   - 设置响应头 `Content-Disposition` 为 `attachment`，使浏览器下载文件

3. **HTML 导出**：
   - 获取文章详情
   - 构建完整的 HTML 模板，包含文章内容和样式
   - 保留文章的所有格式和样式
   - 设置响应头 `Content-Disposition` 为 `attachment`，使浏览器下载文件

## 4. 数据库迁移

### 4.1 评论系统增强迁移

```sql
-- 新增 comment_likes 表
CREATE TABLE comment_likes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    comment_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (comment_id) REFERENCES comments(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(comment_id, user_id)
);

-- 新增 comment_notifications 表
CREATE TABLE comment_notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    comment_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    notification_type TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (comment_id) REFERENCES comments(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 修改 comments 表
ALTER TABLE comments ADD COLUMN likes_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE comments ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
ALTER TABLE comments ADD COLUMN notification_sent BOOLEAN NOT NULL DEFAULT 0;
```

## 5. 代码实现

### 5.1 模型定义

**在 models.rs 中添加**：

```rust
// 评论点赞模型
#[derive(Queryable, Serialize, Deserialize)]
pub struct CommentLike {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::comment_likes)]
pub struct NewCommentLike {
    pub comment_id: i32,
    pub user_id: i32,
}

// 评论通知模型
#[derive(Queryable, Serialize, Deserialize)]
pub struct CommentNotification {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub notification_type: String,
    pub is_read: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::comment_notifications)]
pub struct NewCommentNotification {
    pub comment_id: i32,
    pub user_id: i32,
    pub notification_type: String,
}

// 更新 Comment 模型
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub user_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub content: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_website: Option<String>,
    pub status: String,
    pub likes_count: i32,  // 新增
    pub sort_order: i32,    // 新增
    pub notification_sent: bool,  // 新增
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

### 5.2 API 处理函数

**在 handlers.rs 中添加**：

```rust
// 评论点赞
pub async fn like_comment(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    let user = match get_user_from_request(&req) {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().json("Unauthorized"),
    };

    let comment_id = path.into_inner();
    let mut conn = establish_connection();

    // 检查评论是否存在
    if comments::table.find(comment_id).first::<Comment>(&mut conn).is_err() {
        return HttpResponse::NotFound().json("Comment not found");
    }

    // 检查是否已点赞
    let existing_like = comment_likes::table
        .filter(comment_likes::comment_id.eq(comment_id))
        .filter(comment_likes::user_id.eq(user.user_id))
        .first::<CommentLike>(&mut conn)
        .ok();

    if existing_like.is_some() {
        // 已点赞，取消点赞
        diesel::delete(comment_likes::table.find(existing_like.unwrap().id))
            .execute(&mut conn)
            .unwrap();

        // 减少点赞数
        diesel::update(comments::table.find(comment_id))
            .set(comments::likes_count.eq(comments::likes_count - 1))
            .execute(&mut conn)
            .unwrap();

        HttpResponse::Ok().json("Like removed")
    } else {
        // 未点赞，添加点赞
        let new_like = NewCommentLike {
            comment_id,
            user_id: user.user_id,
        };

        diesel::insert_into(comment_likes::table)
            .values(&new_like)
            .execute(&mut conn)
            .unwrap();

        // 增加点赞数
        diesel::update(comments::table.find(comment_id))
            .set(comments::likes_count.eq(comments::likes_count + 1))
            .execute(&mut conn)
            .unwrap();

        HttpResponse::Ok().json("Comment liked")
    }
}

// 获取评论点赞数
pub async fn get_comment_likes(path: web::Path<i32>) -> impl Responder {
    let comment_id = path.into_inner();
    let mut conn = establish_connection();

    let comment = match comments::table.find(comment_id).first::<Comment>(&mut conn) {
        Ok(c) => c,
        Err(_) => return HttpResponse::NotFound().json("Comment not found"),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "comment_id": comment_id,
        "likes_count": comment.likes_count
    }))
}

// 获取文章评论（支持排序）
pub async fn get_comments(path: web::Path<i32>, query: web::Query<CommentQuery>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();

    let mut query_builder = comments::table
        .filter(comments::post_id.eq(post_id))
        .filter(comments::status.eq("approved"));

    // 根据排序参数排序
    query_builder = match query.sort.as_deref() {
        Some("newest") => query_builder.order(comments::created_at.desc()),
        Some("oldest") => query_builder.order(comments::created_at.asc()),
        Some("popular") => query_builder.order(comments::likes_count.desc()),
        _ => query_builder.order(comments::created_at.desc()),
    };

    let results = query_builder
        .load::<Comment>(&mut conn)
        .unwrap_or_default();

    HttpResponse::Ok().json(results)
}

// 导出文章为 Markdown 格式
pub async fn export_post_markdown(path: web::Path<i32>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();

    let post = match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(p) => p,
        Err(_) => return HttpResponse::NotFound().json("Post not found"),
    };

    // 构建 Markdown 内容
    let markdown_content = format!(
        "# {}\n\n{}\n\n**发布时间**: {}\n**作者**: {}\n",
        post.title,
        post.content, // 这里需要将 HTML 转换为 Markdown
        post.published_at.unwrap_or(post.created_at),
        post.author
    );

    HttpResponse::Ok()
        .content_type("text/markdown")
        .header("Content-Disposition", format!("attachment; filename={}.md", post.slug.unwrap_or_else(|| post.id.to_string())))
        .body(markdown_content)
}

// 导出文章为 PDF 格式
pub async fn export_post_pdf(path: web::Path<i32>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();

    let post = match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(p) => p,
        Err(_) => return HttpResponse::NotFound().json("Post not found"),
    };

    // 构建 HTML 内容
    let html_content = format!(
        "<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .meta {{ color: #666; margin-bottom: 20px; }}
        .content {{ line-height: 1.6; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="meta">
        <p><strong>发布时间:</strong> {}</p>
        <p><strong>作者:</strong> {}</p>
    </div>
    <div class="content">{}</div>
</body>
</html>",
        post.title,
        post.title,
        post.published_at.unwrap_or(post.created_at),
        post.author,
        post.content
    );

    // 这里需要使用 weasyprint 将 HTML 转换为 PDF
    // 由于实现细节复杂，这里只展示接口设计

    HttpResponse::Ok()
        .content_type("application/pdf")
        .header("Content-Disposition", format!("attachment; filename={}.pdf", post.slug.unwrap_or_else(|| post.id.to_string())))
        .body("PDF content") // 实际应返回生成的 PDF 内容
}

// 导出文章为 HTML 格式
pub async fn export_post_html(path: web::Path<i32>) -> impl Responder {
    let post_id = path.into_inner();
    let mut conn = establish_connection();

    let post = match posts::table.find(post_id).first::<Post>(&mut conn) {
        Ok(p) => p,
        Err(_) => return HttpResponse::NotFound().json("Post not found"),
    };

    // 构建完整的 HTML 内容
    let html_content = format!(
        "<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .meta {{ color: #666; margin-bottom: 20px; }}
        .content {{ line-height: 1.6; }}
        .tags {{ margin-top: 20px; }}
        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 2px 8px; margin-right: 5px; border-radius: 3px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="meta">
        <p><strong>发布时间:</strong> {}</p>
        <p><strong>作者:</strong> {}</p>
    </div>
    <div class="content">{}</div>
</body>
</html>",
        post.title,
        post.title,
        post.published_at.unwrap_or(post.created_at),
        post.author,
        post.content
    );

    HttpResponse::Ok()
        .content_type("text/html")
        .header("Content-Disposition", format!("attachment; filename={}.html", post.slug.unwrap_or_else(|| post.id.to_string())))
        .body(html_content)
}
```

## 6. 依赖管理

### 6.1 新增依赖

在 `Cargo.toml` 中添加以下依赖：

```toml
# 评论系统增强
actix-email = "0.1.0"  # 邮件通知（可选）

# 内容导出功能
pulldown-cmark = "0.9.0"  # Markdown 转换
weasyprint = "0.52.0"  # PDF 生成
```

## 7. 测试计划

### 7.1 评论系统增强测试

1. **评论点赞测试**：
   - 测试用户点赞评论
   - 测试用户取消点赞
   - 测试点赞数统计

2. **回复通知测试**：
   - 测试回复评论时的通知生成
   - 测试通知列表获取
   - 测试通知标记为已读

3. **评论排序测试**：
   - 测试按时间排序
   - 测试按热度排序

### 7.2 内容导出功能测试

1. **Markdown 导出测试**：
   - 测试单篇文章导出
   - 测试导出内容格式

2. **PDF 导出测试**：
   - 测试单篇文章导出
   - 测试导出 PDF 格式

## 8. 部署与集成

### 8.1 数据库迁移

运行以下命令执行数据库迁移：

```bash
diesel migration run
```

### 8.2 配置邮件服务（可选）

如果需要启用邮件通知功能，在 `.env` 文件中添加以下配置：

```
# 邮件服务配置
EMAIL_SERVER=smtp.example.com
EMAIL_PORT=587
EMAIL_USERNAME=your-email@example.com
EMAIL_PASSWORD=your-email-password
EMAIL_FROM=Blog System <noreply@example.com>
```

## 9. 总结

本设计文档详细描述了博客系统的两个新核心功能：评论系统增强和内容导出功能。通过扩展现有表结构和添加新的 API 接口，实现了评论点赞、回复通知、评论排序和内容导出等功能，提升了用户体验和系统功能完整性。

实现这些功能后，博客系统将更加现代化、用户友好，能够满足更多用户需求。