# 文章推荐系统设计文档

## 概述

为博客 API 增加基于简单规则的文章推荐系统，记录登录用户的阅读历史，并基于分类、标签、发布时间等因素提供个性化推荐。

## 目标

- 记录用户阅读历史
- 基于简单规则计算文章推荐
- 在获取单篇文章时返回"你可能也喜欢"推荐
- 实现简单、易于维护、易于扩展

## 非目标

- 复杂的机器学习算法
- 匿名用户推荐（后续可扩展）
- 可配置权重（后续可扩展）

---

## 数据模型设计

### 新增表：user_read_history

| 字段 | 类型 | 说明 | 约束 |
|------|------|------|------|
| id | Integer | 主键 | PRIMARY KEY AUTOINCREMENT |
| user_id | Integer | 用户ID | FOREIGN KEY -> users.id, NOT NULL |
| post_id | Integer | 文章ID | FOREIGN KEY -> posts.id, NOT NULL |
| read_at | DateTime | 阅读时间 | NOT NULL, DEFAULT CURRENT_TIMESTAMP |
| read_duration | Integer | 阅读时长（秒） | NULLABLE |

### 索引

- `idx_user_post`: (user_id, post_id) - 用于去重查询
- `idx_user_read_at`: (user_id, read_at DESC) - 用于获取最近阅读

### 模型结构

在 `src/models.rs` 中新增：

```rust
#[derive(Queryable, Serialize, Deserialize)]
pub struct UserReadHistory {
    pub id: i32,
    pub user_id: i32,
    pub post_id: i32,
    pub read_at: chrono::NaiveDateTime,
    pub read_duration: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_read_history)]
pub struct NewUserReadHistory {
    pub user_id: i32,
    pub post_id: i32,
    pub read_duration: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct RecommendedPost {
    pub id: i32,
    pub title: String,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub cover_image: Option<String>,
    pub author: String,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub category_name: Option<String>,
    pub tag_names: Vec<String>,
    pub view_count: Option<i32>,
    pub relevance_score: i32,
}
```

---

## 推荐算法设计

### 权重配置

| 因素 | 权重 | 说明 |
|------|------|------|
| 相同分类 | 50 | 最重要的匹配因素 |
| 相同标签 | 10/个 | 每个相同标签加 10 分，最多 30 分 |
| 近期发布 | 20 | 最近 30 天内发布的文章 |

### 计算流程

1. 获取所有已发布文章（排除当前文章）
2. 对每篇候选文章计算得分：
   ```
   得分 = (同分类 ? 50 : 0) + 
          (相同标签数 * 10, 最高30) + 
          (30天内发布 ? 20 : 0)
   ```
3. 排除用户已读过的文章
4. 按得分降序排序，取前 6 篇

### 阅读记录触发

- 在 `get_post` handler 中，若用户已登录，自动记录阅读历史
- 同一用户对同一文章的多次阅读，更新最近阅读时间（不创建重复记录）

---

## API 接口设计

### 1. 增强现有接口

**GET /api/posts/{id}**

- 登录用户：返回文章详情 + 推荐文章
- 匿名用户：仅返回文章详情（保持向后兼容）

**响应示例：**

```json
{
  "post": {
    "id": 1,
    "title": "文章标题",
    "content": "文章内容...",
    "...": "其他字段"
  },
  "recommendations": [
    {
      "id": 123,
      "title": "推荐文章标题",
      "slug": "recommended-post",
      "excerpt": "文章摘要",
      "cover_image": "图片URL",
      "author": "作者名",
      "published_at": "2026-04-20T10:00:00",
      "category_name": "分类名",
      "tag_names": ["标签1", "标签2"],
      "view_count": 156,
      "relevance_score": 80
    }
  ]
}
```

### 2. 新增推荐接口

**GET /api/posts/{id}/recommendations**

独立获取推荐文章的接口，用于前端单独加载推荐区域。

**响应示例：**

```json
{
  "recommendations": [
    /* 同上 */
  ]
}
```

### 3. 阅读历史管理接口

**GET /api/users/me/read-history**

获取当前用户的阅读历史（按时间倒序，支持分页）。

**DELETE /api/users/me/read-history/{post_id}**

删除单条阅读记录。

**DELETE /api/users/me/read-history**

清空所有阅读历史。

---

## 实现步骤

### 阶段 1：数据库迁移
- 创建 user_read_history 表迁移
- 运行迁移

### 阶段 2：数据模型
- 在 schema.rs 添加表定义
- 在 models.rs 添加结构体

### 阶段 3：核心逻辑
- 实现阅读记录函数
- 实现推荐计算函数

### 阶段 4：API 集成
- 修改 get_post handler
- 添加新的推荐接口
- 添加阅读历史管理接口

### 阶段 5：测试
- 单元测试推荐算法
- 集成测试 API 接口

---

## 技术考虑

### 性能
- 使用数据库索引优化查询
- 推荐计算在应用层完成，避免复杂 SQL
- 考虑后续添加缓存（暂不实现）

### 扩展性
- 权重配置后续可移至数据库或配置文件
- 算法逻辑模块化，便于未来替换为更复杂的算法
- 为匿名用户推荐预留扩展空间

### 兼容性
- 保持现有 API 接口不变
- 匿名用户不受影响

---

## 后续可能的扩展

1. 支持匿名用户推荐（基于 cookie/session）
2. 可配置权重和推荐数量
3. 添加用户偏好管理
4. 更复杂的推荐算法（协同过滤等）
5. 推荐结果缓存
6. A/B 测试框架
