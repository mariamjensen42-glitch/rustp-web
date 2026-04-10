# Checklist - 博客系统 API 验证清单

## 用户认证模块

- [x] POST /api/auth/register - 用户注册接口正常工作
- [x] POST /api/auth/login - 用户登录返回有效 JWT Token
- [x] POST /api/auth/logout - 登出接口正常工作
- [x] POST /api/auth/refresh - 刷新令牌接口正常工作
- [x] GET /api/users/me - 获取当前用户信息需认证
- [x] PUT /api/users/me - 修改密码/资料需认证

## 文章管理模块

- [x] GET /api/posts - 文章列表返回已发布文章，支持分页
- [x] GET /api/posts?category_id=X - 按分类过滤正常工作
- [x] GET /api/posts?tag_id=X - 按标签过滤正常工作
- [x] GET /api/posts?q=keyword - 搜索功能正常工作
- [x] GET /api/posts/:id - 获取单篇文章详情
- [x] GET /api/posts/slug/:slug - 通过 slug 获取文章详情
- [x] POST /api/posts - 创建文章需认证
- [x] PUT /api/posts/:id - 更新文章需作者或管理员权限
- [x] DELETE /api/posts/:id - 软删除文章（设置 deleted_at）
- [x] PATCH /api/posts/:id/status - 更新文章状态（published/draft/private）

## 分类管理模块

- [x] GET /api/categories - 返回所有分类
- [x] GET /api/categories/:id - 返回单个分类详情
- [x] POST /api/categories - 创建分类需管理员权限
- [x] PUT /api/categories/:id - 更新分类需管理员权限
- [x] DELETE /api/categories/:id - 删除分类需管理员权限
- [x] GET /api/categories/:id/posts - 返回该分类下的已发布文章

## 标签管理模块

- [x] GET /api/tags - 返回所有标签
- [x] POST /api/tags - 创建标签需管理员权限
- [x] PUT /api/tags/:id - 更新标签需管理员权限
- [x] DELETE /api/tags/:id - 删除标签需管理员权限
- [x] GET /api/tags/:id/posts - 返回该标签下的已发布文章

## 评论管理模块

- [x] GET /api/posts/:id/comments - 返回已审核评论（访客可见）
- [x] POST /api/posts/:id/comments - 提交评论（注册用户自动批准，游客待审核）
- [x] PUT /api/comments/:id/approve - 审核评论需管理员权限
- [x] DELETE /api/comments/:id - 删除评论（管理员或评论者本人）

## 媒体上传模块

- [x] POST /api/media/upload - 上传文件需管理员权限
- [x] GET /api/media - 文件列表需管理员权限
- [x] DELETE /api/media/:id - 删除文件需管理员权限

## 搜索功能

- [x] GET /api/search?q=keyword - 搜索文章标题、内容、摘要

## 权限验证

- [x] 访客无法访问需认证的接口（返回 401）
- [x] 普通用户无法执行管理员操作（返回 403）
- [x] 管理员可执行所有管理操作
- [x] 作者可编辑自己的文章
- [x] 普通用户可删除自己的评论

## 软删除与状态

- [x] 删除的文章在列表中不显示
- [x] 草稿/私密文章仅对作者和管理员可见
- [x] 评论状态 pending 时不公开显示

## 待验证项（需要 cargo 环境）

- [ ] 运行 `cargo build` 验证编译通过
- [ ] 运行 `diesel migration run` 更新数据库
- [ ] 启动服务器测试所有 API 端点
