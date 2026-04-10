# 博客系统 API 规范

## Why
需要构建一个功能完整的博客后端 API 系统，支持用户认证、文章管理、分类标签、评论、媒体上传和搜索功能。

## What Changes

### 1. 用户认证与权限管理
- **注册**：`POST /api/auth/register` - 访客注册（可关闭仅管理员创建）
- **登录**：`POST /api/auth/login` - 返回 JWT Token
- **登出**：`POST /api/auth/logout`
- **刷新令牌**：`POST /api/auth/refresh`
- **获取当前用户**：`GET /api/users/me`
- **修改密码/资料**：`PUT /api/users/me`

### 2. 文章管理（核心）
- **获取文章列表**：`GET /api/posts` - 支持分页、排序、分类/标签过滤、搜索
- **获取文章详情**：`GET /api/posts/:id` 或 `GET /api/posts/:slug`
- **创建文章**：`POST /api/posts` - 仅管理员/作者
- **更新文章**：`PUT /api/posts/:id`
- **删除文章**：`DELETE /api/posts/:id` - 软删除
- **更新文章状态**：`PATCH /api/posts/:id/status` - 发布/草稿/私密

### 3. 分类管理
- **获取分类列表**：`GET /api/categories`
- **获取分类详情**：`GET /api/categories/:id`
- **创建分类**：`POST /api/categories` - 管理员
- **更新分类**：`PUT /api/categories/:id`
- **删除分类**：`DELETE /api/categories/:id`
- **获取分类下文章**：`GET /api/categories/:id/posts`

### 4. 标签管理
- **获取标签列表**：`GET /api/tags`
- **创建标签**：`POST /api/tags`
- **更新标签**：`PUT /api/tags/:id`
- **删除标签**：`DELETE /api/tags/:id`
- **获取标签下文章**：`GET /api/tags/:id/posts`

### 5. 评论管理
- **获取文章评论**：`GET /api/posts/:id/comments` - 公开，仅展示已审核评论
- **提交评论**：`POST /api/posts/:id/comments` - 防垃圾：频率限制
- **审核评论**：`PUT /api/comments/:id/approve` - 管理员
- **删除评论**：`DELETE /api/comments/:id` - 管理员或评论者本人

### 6. 媒体/文件管理
- **上传文件**：`POST /api/media/upload` - 支持图片
- **获取文件列表**：`GET /api/media` - 管理员
- **删除文件**：`DELETE /api/media/:id`

### 7. 搜索功能
- **全局搜索**：`GET /api/search?q=keyword` - 搜索文章标题、内容、摘要

## Impact
- Affected specs: 用户认证、文章 CRUD、分类、标签、评论、媒体上传、全文搜索
- Affected code: src/main.rs, src/auth.rs, src/handlers.rs, src/models.rs, src/schema.rs

## ADDED Requirements

### Requirement: JWT 认证
系统 SHALL 使用 JWT Token 进行用户身份验证，Token 包含用户 ID 和角色信息。

#### Scenario: 有效 Token 请求
- **WHEN** 用户携带有效 JWT Token 发送请求
- **THEN** 系统返回请求的资源

#### Scenario: 无效/过期 Token 请求
- **WHEN** 用户携带无效或过期 JWT Token 发送请求
- **THEN** 系统返回 401 Unauthorized

### Requirement: 用户角色权限
系统 SHALL 支持两种角色：管理员（admin）和普通用户（user）。

#### Scenario: 管理员操作
- **WHEN** 管理员执行管理操作（创建分类、审核评论等）
- **THEN** 系统允许操作并返回成功

#### Scenario: 普通用户尝试管理员操作
- **WHEN** 普通用户尝试执行管理员专属操作
- **THEN** 系统返回 403 Forbidden

### Requirement: 文章状态管理
系统 SHALL 支持三种文章状态：发布（published）、草稿（draft）、私密（private）。

#### Scenario: 获取已发布文章
- **WHEN** 访客请求文章列表
- **THEN** 系统仅返回已发布文章

#### Scenario: 获取草稿/私密文章
- **WHEN** 作者请求自己的草稿/私密文章
- **THEN** 系统返回对应文章

### Requirement: 评论审核机制
系统 SHALL 支持评论审核，访客提交的评论需管理员审核后才能展示。

#### Scenario: 提交新评论
- **WHEN** 访客提交评论
- **THEN** 评论状态为 pending（待审核）

#### Scenario: 审核评论
- **WHEN** 管理员审核通过评论
- **THEN** 评论状态变为 approved，可在文章评论列表中展示

### Requirement: 全局搜索
系统 SHALL 支持全文搜索文章标题、内容和摘要。

#### Scenario: 搜索文章
- **WHEN** 用户访问 `GET /api/search?q=keyword`
- **THEN** 系统返回标题、内容或摘要包含 keyword 的已发布文章

### Requirement: 软删除
系统 SHALL 支持文章软删除，删除后文章不显示但数据保留。

#### Scenario: 删除文章
- **WHEN** 管理员删除文章
- **THEN** 文章的 `deleted_at` 字段被设置，查询时排除

### Requirement: 分页与过滤
系统 SHALL 支持文章列表的分页、排序和分类/标签过滤。

#### Scenario: 分页获取文章
- **WHEN** 请求 `GET /api/posts?page=1&per_page=10`
- **THEN** 系统返回第 1 页，每页 10 篇文章

#### Scenario: 按分类过滤
- **WHEN** 请求 `GET /api/posts?category_id=1`
- **THEN** 系统仅返回该分类下的已发布文章

## MODIFIED Requirements

### Requirement: 数据库迁移
现有数据库表结构需要通过 Diesel 迁移管理，确保 schema 与规范一致。

## REMOVED Requirements

（无）

## 技术实现

### 数据库表结构
- `users`: id, username, email, password_hash, role, created_at, updated_at
- `posts`: id, title, slug, content, excerpt, status, author_id, published_at, created_at, updated_at, deleted_at
- `categories`: id, name, slug, description, created_at, updated_at
- `tags`: id, name, slug, created_at, updated_at
- `post_tags`: post_id, tag_id
- `comments`: id, post_id, user_id, content, status, created_at, updated_at
- `media`: id, filename, filepath, mimetype, size, uploaded_by, created_at

### 认证流程
1. 用户登录获取 JWT Token（有效期 24 小时）
2. 后续请求携带 `Authorization: Bearer <token>`
3. Token 过期后使用 refresh token 刷新

### 角色权限矩阵
| 操作 | 访客 | 注册用户 | 管理员 |
|------|------|----------|--------|
| 查看已发布文章 | ✓ | ✓ | ✓ |
| 提交评论 | - | ✓ | ✓ |
| 创建文章 | - | ✓ (本人) | ✓ |
| 管理分类/标签 | - | - | ✓ |
| 审核评论 | - | - | ✓ |
| 上传媒体 | - | - | ✓ |
| 删除任意文章 | - | - | ✓ |
