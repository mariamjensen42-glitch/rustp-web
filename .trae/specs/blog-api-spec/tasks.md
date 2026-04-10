# Tasks - 博客系统 API 实现

## 任务列表

- [x] Task 1: 完善数据库 Schema 与模型
  - [x] SubTask 1.1: 检查并更新 users 表结构（添加 role 字段）
  - [x] SubTask 1.2: 检查并更新 posts 表结构（添加 slug, excerpt, status, deleted_at）
  - [x] SubTask 1.3: 检查 media 表是否存在，如不存在则创建迁移
  - [x] SubTask 1.4: 在 models.rs 中完善所有数据模型

- [x] Task 2: 实现用户认证模块
  - [x] SubTask 2.1: 实现 JWT Token 生成与验证（auth.rs）
  - [x] SubTask 2.2: 实现 POST /api/auth/register 注册接口
  - [x] SubTask 2.3: 实现 POST /api/auth/login 登录接口
  - [x] SubTask 2.4: 实现 POST /api/auth/logout 登出接口
  - [x] SubTask 2.5: 实现 POST /api/auth/refresh 刷新令牌接口
  - [x] SubTask 2.6: 实现 GET /api/users/me 获取当前用户
  - [x] SubTask 2.7: 实现 PUT /api/users/me 修改密码/资料

- [x] Task 3: 实现文章管理模块
  - [x] SubTask 3.1: 实现 GET /api/posts 文章列表（分页、排序、过滤）
  - [x] SubTask 3.2: 实现 GET /api/posts/:id 或 /slug 文章详情
  - [x] SubTask 3.3: 实现 POST /api/posts 创建文章（需认证）
  - [x] SubTask 3.4: 实现 PUT /api/posts/:id 更新文章
  - [x] SubTask 3.5: 实现 DELETE /api/posts/:id 软删除文章
  - [x] SubTask 3.6: 实现 PATCH /api/posts/:id/status 更新文章状态

- [x] Task 4: 实现分类管理模块
  - [x] SubTask 4.1: 实现 GET /api/categories 分类列表
  - [x] SubTask 4.2: 实现 GET /api/categories/:id 分类详情
  - [x] SubTask 4.3: 实现 POST /api/categories 创建分类（管理员）
  - [x] SubTask 4.4: 实现 PUT /api/categories/:id 更新分类
  - [x] SubTask 4.5: 实现 DELETE /api/categories/:id 删除分类
  - [x] SubTask 4.6: 实现 GET /api/categories/:id/posts 获取分类下文章

- [x] Task 5: 实现标签管理模块
  - [x] SubTask 5.1: 实现 GET /api/tags 标签列表
  - [x] SubTask 5.2: 实现 POST /api/tags 创建标签
  - [x] SubTask 5.3: 实现 PUT /api/tags/:id 更新标签
  - [x] SubTask 5.4: 实现 DELETE /api/tags/:id 删除标签
  - [x] SubTask 5.5: 实现 GET /api/tags/:id/posts 获取标签下文章

- [x] Task 6: 实现评论管理模块
  - [x] SubTask 6.1: 实现 GET /api/posts/:id/comments 获取文章评论
  - [x] SubTask 6.2: 实现 POST /api/posts/:id/comments 提交评论
  - [x] SubTask 6.3: 实现 PUT /api/comments/:id/approve 审核评论
  - [x] SubTask 6.4: 实现 DELETE /api/comments/:id 删除评论

- [x] Task 7: 实现媒体上传模块
  - [x] SubTask 7.1: 实现 POST /api/media/upload 上传文件
  - [x] SubTask 7.2: 实现 GET /api/media 文件列表（管理员）
  - [x] SubTask 7.3: 实现 DELETE /api/media/:id 删除文件

- [x] Task 8: 实现搜索功能
  - [x] SubTask 8.1: 实现 GET /api/search 全局搜索

- [x] Task 9: 配置路由与中间件
  - [x] SubTask 9.1: 配置 CORS 中间件
  - [x] SubTask 9.2: 配置认证中间件
  - [x] SubTask 9.3: 挂载所有 API 路由

- [ ] Task 10: 测试与验证
  - [ ] SubTask 10.1: 编写单元测试
  - [ ] SubTask 10.2: 验证所有 API 端点

## 任务依赖

- Task 3 依赖于 Task 1、Task 2
- Task 4 依赖于 Task 1
- Task 5 依赖于 Task 1
- Task 6 依赖于 Task 1、Task 2、Task 3
- Task 7 依赖于 Task 1、Task 2
- Task 8 依赖于 Task 1、Task 3
- Task 9 依赖于 Task 2、Task 3、Task 4、Task 5、Task 6、Task 7、Task 8
- Task 10 依赖于 Task 9
