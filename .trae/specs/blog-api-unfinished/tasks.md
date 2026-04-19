# Blog API 未完成功能 - 实施计划

## [ ] 任务 1: 完善媒体上传功能
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 取消注释 handlers.rs 中的媒体功能代码
  - 修复并完善 upload_media、get_media、delete_media 函数
  - 添加文件类型验证（只允许图片）
  - 添加文件大小限制（10MB）
  - 确保文件保存到正确的目录
  - 修复错误处理
- **Acceptance Criteria Addressed**: AC-1, AC-2, AC-3
- **Test Requirements**:
  - `programmatic` 能成功上传合法图片文件
  - `programmatic` 能拒绝非法文件类型
  - `programmatic` 能拒绝过大的文件
  - `programmatic` 能获取媒体列表
  - `programmatic` 能删除媒体文件
- **Notes**: 需要确保 uploads 目录正确创建

## [ ] 任务 2: 添加静态文件服务
- **Priority**: P0
- **Depends On**: 任务 1
- **Description**: 
  - 配置 Actix-web 提供 uploads 目录的静态文件服务
  - 添加路由用于访问已上传的文件
- **Acceptance Criteria Addressed**: AC-1, AC-2
- **Test Requirements**:
  - `programmatic` 能通过 URL 访问已上传的文件
- **Notes**: 用于访问图片文件

## [ ] 任务 3: 启用媒体路由
- **Priority**: P0
- **Depends On**: 任务 1, 任务 2
- **Description**: 
  - 取消注释 main.rs 中与媒体相关的导入
  - 取消注释 main.rs 中的媒体路由
- **Acceptance Criteria Addressed**: AC-1, AC-2, AC-3
- **Test Requirements**:
  - `programmatic` 能访问媒体相关的 API 端点

## [ ] 任务 4: 实现自动调度发布功能
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 创建一个后台任务，定期检查待发布的文章
  - 当到达预定时间时，自动更新文章状态为已发布
  - 在 main.rs 中启动定时任务
  - 使用 tokio 的定时功能实现
- **Acceptance Criteria Addressed**: AC-4
- **Test Requirements**:
  - `programmatic` 定时任务能正常启动
  - `programmatic` 到达时间的文章能自动发布

## [ ] 任务 5: 实现用户头像上传功能
- **Priority**: P0
- **Depends On**: 任务 1, 任务 2
- **Description**: 
  - 添加新的处理函数用于上传头像
  - 头像上传时更新用户的 avatar 字段
  - 添加新的路由
- **Acceptance Criteria Addressed**: AC-5
- **Test Requirements**:
  - `programmatic` 能上传用户头像
  - `programmatic` 上传后用户头像字段正确更新

## [ ] 任务 6: 编译和测试整个应用
- **Priority**: P0
- **Depends On**: 任务 1-5
- **Description**: 
  - 编译项目，确保无编译错误
  - 测试所有新增功能
- **Acceptance Criteria Addressed**: 所有
- **Test Requirements**:
  - `programmatic` 项目无编译错误
  - `human-judgment` 所有 API 功能正常工作
