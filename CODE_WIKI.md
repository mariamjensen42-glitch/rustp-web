# Blog API - Code Wiki

## 项目概述

这是一个功能完整的博客系统后端 API，使用 Rust 和 Actix-web 框架开发，提供了完整的博客内容管理、用户认证、评论系统和数据分析功能。

### 技术栈

| 技术               | 用途                             |
|--------------------|----------------------------------|
| Rust               | 主要开发语言                     |
| Actix-web          | Web 框架                        |
| Diesel             | ORM 和数据库操作                |
| SQLite             | 数据库                         |
| jsonwebtoken       | JWT 认证                        |
| bcrypt             | 密码加密                        |
| actix-cors         | 跨域处理                        |
| validator          | 数据验证                        |

---

## 目录结构

```
/workspace/
├── migrations/         # 数据库迁移文件
│   └── *.sql/         # 迁移脚本
├── src/               # 源代码
│   ├── main.rs        # 应用入口和路由配置
│   ├── models.rs      # 数据库模型定义
│   ├── schema.rs      # Diesel 数据库 schema
│   ├── handlers.rs    # API 处理函数
│   ├── auth.rs        # 认证相关功能
│   └── db.rs          # 数据库连接
├── .env               # 环境变量
├── Cargo.toml         # Cargo 项目配置
├── diesel.toml        # Diesel 配置
└── README.md          # 项目说明
```

---

## 核心模块说明

### 1. main.rs - 应用入口

[main.rs](file:///workspace/src/main.rs) 负责启动应用服务器和配置所有 API 路由。

**主要功能：**
- 初始化 `AppState`（Token 黑名单管理）
- 配置 CORS 中间件（允许所有来源）
- 定义所有 REST API 端点路由

### 2. models.rs - 数据模型

[models.rs](file:///workspace/src/models.rs) 定义了所有数据库实体模型和相关类型。

**核心模型：**

| 模型          | 描述                                      |
|---------------|-------------------------------------------|
| `Post`        | 博客文章，包含标题、内容、状态、发布时间等  |
| `User`        | 用户信息，包含用户名、邮箱、角色等         |
| `Category`    | 分类目录                                  |
| `Tag`         | 标签                                      |
| `Comment`     | 文章评论，支持层级回复                     |
| `PostVersion` | 文章版本历史，用于回滚功能                 |
| `PostAnalytic`| 文章访问分析数据                           |
| `Media`       | 媒体文件（功能待实现）                     |

### 3. handlers.rs - API 处理

[handlers.rs](file:///workspace/src/handlers.rs) 是最大的模块，包含所有 API 处理函数和请求响应结构。

**主要功能分组：**
- **认证**：注册、登录、登出、Token 刷新
- **用户**：获取当前用户、更新资料、修改密码
- **文章**：CRUD、状态更新、标签管理、草稿、版本控制
- **分类**：分类管理和获取分类文章
- **标签**：标签管理和获取标签文章
- **评论**：评论的发表、审核、删除
- **搜索**：文章搜索
- **高级功能**：相关文章、调度发布、分析数据

### 4. auth.rs - 认证模块

[auth.rs](file:///workspace/src/auth.rs) 负责 JWT Token 生成和验证，以及密码哈希。

**主要函数：**
| 函数                | 描述                                  |
|---------------------|---------------------------------------|
| `generate_token`    | 生成 24 小时有效期的 JWT Token        |
| `verify_token`      | 验证并解析 Token                      |
| `hash_password`     | 使用 bcrypt 加密密码                  |
| `verify_password`   | 验证密码                              |

**Claims 结构：**
- `user_id`: 用户 ID
- `username`: 用户名
- `role`: 用户角色（`user` 或 `admin`）
- `exp`: 过期时间戳

### 5. db.rs - 数据库连接

[db.rs](file:///workspace/src/db.rs) 提供数据库连接建立函数。

### 6. schema.rs - 数据库 Schema

[schema.rs](file:///workspace/src/schema.rs) 由 Diesel 自动生成，定义了所有数据库表和关系。

---

## 数据库设计

### 核心表关系

```
users ──┬─── posts (user_id)
        │
posts ──┼─── comments (post_id)
        ├─── post_tags (post_id ↔ tag_id)
        ├─── post_versions (post_id)
        ├─── post_analytics (post_id)
        └─── categories (category_id)
        
comments ─── comments (parent_id - 嵌套回复)
```

### 表详细说明

#### 1. users 表

| 字段            | 类型      | 描述           |
|----------------|----------|----------------|
| id             | Integer  | 主键           |
| username       | Text     | 用户名         |
| email          | Text     | 邮箱           |
| password_hash  | Text     | 密码哈希       |
| role           | Text     | 角色           |
| avatar         | Text     | 头像（可选）   |
| bio            | Text     | 简介（可选）   |
| created_at     | Timestamp| 创建时间       |
| updated_at     | Timestamp| 更新时间       |

#### 2. posts 表

| 字段            | 类型       | 描述           |
|----------------|----------|----------------|
| id             | Integer  | 主键           |
| title          | Text     | 标题           |
| slug           | Text     | URL 友好标识  |
| content        | Text     | 文章内容       |
| excerpt        | Text     | 摘要           |
| author         | Text     | 作者名         |
| status         | Text     | 状态（draft/published/scheduled） |
| created_at     | Timestamp| 创建时间       |
| updated_at     | Timestamp| 更新时间       |
| published_at   | Timestamp| 发布时间       |
| deleted_at     | Timestamp| 软删除时间     |
| category_id    | Integer  | 分类 ID        |
| user_id        | Integer  | 作者 ID        |
| summary        | Text     | 总结           |
| cover_image    | Text     | 封面图         |
| is_published   | Bool     | 是否已发布     |
| is_top         | Bool     | 是否置顶       |
| allow_comments | Bool     | 是否允许评论   |
| view_count     | Integer  | 浏览次数       |
| scheduled_at   | Timestamp| 调度发布时间   |
| is_scheduled   | Bool     | 是否调度发布   |
| draft_saved_at | Timestamp| 草稿保存时间   |
| auto_save_draft| Bool     | 是否自动保存   |

#### 3. 其他表
- **categories**: 分类表，支持树状结构（parent_id）
- **tags**: 标签表
- **post_tags**: 文章-标签关联表
- **comments**: 评论表，支持回复（parent_id）
- **post_versions**: 文章版本历史表
- **post_analytics**: 文章访问分析表（按天统计）
- **media**: 媒体文件表（功能待实现）

---

## API 接口文档

### 基础信息

- **Base URL**: `http://localhost:8080`
- **认证方式**: Bearer Token（JWT）
- **响应格式**: JSON

---

### 1. 认证接口 (/api/auth)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| POST | `/register` | 用户注册 | ❌ |
| POST | `/login` | 用户登录 | ❌ |
| POST | `/logout` | 用户登出 | ✅ |
| POST | `/refresh` | 刷新 Token | ❌ |

#### 注册请求示例
```json
POST /api/auth/register
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```

#### 登录请求示例
```json
POST /api/auth/login
{
  "email": "test@example.com",
  "password": "password123"
}

// 响应
{
  "token": "eyJ0eXA...",
  "user": {
    "id": 1,
    "username": "testuser",
    "email": "test@example.com",
    "role": "user"
  }
}
```

---

### 2. 用户接口 (/api/users)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/me` | 获取当前用户信息 | ✅ |
| PUT | `/me` | 更新当前用户资料 | ✅ |
| PUT | `/me/password` | 修改密码 | ✅ |

---

### 3. 文章接口 (/api/posts)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/` | 获取文章列表（支持分页、筛选） | ❌ |
| GET | `/{id}` | 获取单篇文章 | ❌ |
| GET | `/slug/{slug}` | 通过 slug 获取文章 | ❌ |
| POST | `/` | 创建文章 | ✅ |
| PUT | `/{id}` | 更新文章 | ✅（作者或 admin） |
| DELETE | `/{id}` | 删除文章（软删除） | ✅（作者或 admin） |
| PATCH | `/{id}/status` | 更新文章状态 | ✅（作者或 admin） |
| GET | `/{id}/comments` | 获取文章评论 | ❌ |
| POST | `/{id}/comments` | 发表评论 | ❌/✅ |
| POST | `/tags` | 为文章添加标签 | ✅ |
| GET | `/{id}/versions` | 获取文章版本历史 | ✅（作者或 admin） |
| GET | `/{id}/versions/{version_number}` | 获取特定版本 | ✅（作者或 admin） |
| POST | `/{id}/rollback` | 回滚到特定版本 | ✅（作者或 admin） |
| POST | `/{id}/schedule` | 调度发布 | ✅（作者或 admin） |
| GET | `/{id}/analytics` | 获取文章分析数据 | ✅（作者或 admin） |
| GET | `/{id}/related` | 获取相关文章 | ❌ |
| POST | `/{id}/draft` | 保存草稿 | ✅ |
| GET | `/drafts` | 获取草稿列表 | ✅ |

#### 文章列表查询参数
| 参数 | 类型 | 描述 |
|-----|-----|------|
| page | Integer | 页码（默认 1） |
| per_page | Integer | 每页数量（默认 10） |
| category_id | Integer | 分类 ID 筛选 |
| tag_id | Integer | 标签 ID 筛选 |
| q | String | 搜索关键词 |

---

### 4. 分类接口 (/api/categories)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/` | 获取所有分类 | ❌ |
| GET | `/{id}` | 获取单个分类 | ❌ |
| POST | `/` | 创建分类 | ✅（仅 admin） |
| PUT | `/{id}` | 更新分类 | ✅（仅 admin） |
| DELETE | `/{id}` | 删除分类 | ✅（仅 admin） |
| GET | `/{id}/posts` | 获取分类下的文章 | ❌ |

---

### 5. 标签接口 (/api/tags)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/` | 获取所有标签 | ❌ |
| POST | `/` | 创建标签 | ✅（仅 admin） |
| PUT | `/{id}` | 更新标签 | ✅（仅 admin） |
| DELETE | `/{id}` | 删除标签 | ✅（仅 admin） |
| GET | `/{id}/posts` | 获取标签下的文章 | ❌ |

---

### 6. 评论接口 (/api/comments)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| PUT | `/{id}/approve` | 审核评论 | ✅（仅 admin） |
| DELETE | `/{id}` | 删除评论 | ✅（作者或 admin） |

---

### 7. 搜索接口 (/api/search)

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/` | 搜索文章（标题/内容/摘要） | ❌ |

查询参数：`q=搜索关键词`

---

## 依赖管理

### Cargo.toml 依赖

[Cargo.toml](file:///workspace/Cargo.toml) 定义了所有项目依赖：

```toml
[dependencies]
actix-web = "4.7.0"
actix-rt = "2.9.0"
actix-multipart = "0.7.0"
diesel = { version = "2.1.0", features = ["sqlite", "chrono"] }
diesel_migrations = "2.1.0"
tokio = { version = "1.37.0", features = ["full"] }
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
dotenv = "0.15.0"
chrono = { version = "0.4.38", features = ["serde"] }
jsonwebtoken = "9.3.0"
bcrypt = "0.15.0"
actix-cors = "0.7.0"
validator = { version = "0.16.0", features = ["derive"] }
```

---

## 运行说明

### 1. 环境变量配置

在 [.env](file:///workspace/.env) 中配置：
```
DATABASE_URL=./blog.db
```

### 2. 数据库迁移

确保数据库 schema 是最新的：
```bash
diesel migration run
```

### 3. 运行应用

开发模式：
```bash
cargo run
```

生产构建：
```bash
cargo build --release
```

### 4. 访问服务

服务默认运行在：`http://localhost:8080`

---

## 安全注意事项

⚠️ **生产环境注意：**

1. **JWT 密钥**：当前硬编码为 "your-secret-key"，生产中必须使用强密钥并通过环境变量配置
2. **CORS**：当前允许所有来源，生产中应配置具体白名单
3. **数据库**：SQLite 适合开发，生产环境应考虑 PostgreSQL 或 MySQL
4. **HTTPS**：生产环境必须使用 HTTPS
5. **Token 过期**：设置合理的过期时间（当前 24 小时）

---

## 待实现功能

1. **媒体上传**：[handlers.rs](file:///workspace/src/handlers.rs#L997-L1082) 中的媒体功能已注释，待完善
2. **自动调度发布**：当前调度功能只是标记，实际需要定时任务检查并自动发布
3. **用户头像上传**
4. **邮件通知**：评论回复通知等
5. **缓存**：热门文章缓存，减少数据库压力
6. **搜索优化**：集成全文搜索引擎（如 Meilisearch 或 Elasticsearch）

---

## 总结

这是一个架构清晰、功能完整的博客后端系统，采用了 Rust 生态中的最佳实践，代码组织合理，可扩展性强。主要特点包括完整的 CRUD 操作、用户认证、内容版本控制、调度发布、数据分析等企业级功能。
