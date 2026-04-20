# 推荐系统实现检查清单

## 数据库层

- [ ] 创建 user_read_history 表的迁移文件（up.sql）
- [ ] 创建回滚迁移文件（down.sql）
- [ ] 添加必要的索引
- [ ] 运行迁移验证表结构

## 数据模型层

- [ ] 在 schema.rs 中添加 user_read_history 表定义
- [ ] 在 models.rs 中添加 UserReadHistory 结构体
- [ ] 在 models.rs 中添加 NewUserReadHistory 结构体
- [ ] 在 models.rs 中添加 RecommendedPost 结构体

## 核心逻辑层

- [ ] 实现记录阅读历史的函数
- [ ] 实现获取用户已读文章 ID 列表的函数
- [ ] 实现获取文章标签的辅助函数
- [ ] 实现推荐算法计算函数
- [ ] 测试推荐算法逻辑

## API 层

- [ ] 修改 get_post handler，添加阅读记录和推荐
- [ ] 添加 GET /api/posts/{id}/recommendations 接口
- [ ] 添加 GET /api/users/me/read-history 接口
- [ ] 添加 DELETE /api/users/me/read-history/{post_id} 接口
- [ ] 添加 DELETE /api/users/me/read-history 接口
- [ ] 在 main.rs 中注册新路由

## 集成测试

- [ ] 测试阅读历史记录
- [ ] 测试推荐结果返回
- [ ] 测试阅读历史管理接口
- [ ] 验证匿名用户行为不受影响
