# 📝 Blog API - 博客系统后端

<div align="center">

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=flat-square&logo=rust)
![Actix Web](https://img.shields.io/badge/Actix%20Web-4.7.0-blue?style=flat-square&logo=actix-web)
![Diesel](https://img.shields.io/badge/Diesel-2.1.0-red?style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-3.35+-blue?style=flat-square&logo=sqlite)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)
![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen?style=flat-square)
![Version](https://img.shields.io/badge/Version-0.1.0-blue?style=flat-square)
![Maintenance](https://img.shields.io/badge/Maintained-Yes-green?style=flat-square)
![PRs Welcome](https://img.shields.io/badge/PRs-Welcome-brightgreen?style=flat-square)

一个功能完整的博客系统后端 API，使用 Rust 和 Actix-web 框架开发，提供用户认证、文章管理、评论系统、标签分类、版本控制和数据分析等功能。

[项目文档](#-项目文档) · [功能特性](#-功能特性) · [快速开始](#-快速开始) · [API 文档](#-api-文档)

</div>

---

## 📦 项目徽章

| 类别 | 徽章 |
|------|------|
| **技术栈** | ![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?style=for-the-badge&logo=rust) ![Actix Web](https://img.shields.io/badge/Actix%20Web-4.7.0-blue?style=for-the-badge&logo=actix-web) |
| **数据库** | ![SQLite](https://img.shields.io/badge/SQLite-3.35+-blue?style=for-the-badge&logo=sqlite) ![Diesel](https://img.shields.io/badge/Diesel-2.1.0-red?style=for-the-badge) |
| **认证** | ![JWT](https://img.shields.io/badge/JWT-Auth-000000?style=for-the-badge&logo=json-web-tokens) ![bcrypt](https://img.shields.io/badge/bcrypt-Password%20Hashing-green?style=for-the-badge) |
| **开发工具** | ![Cargo](https://img.shields.io/badge/Cargo-Package%20Manager-orange?style=for-the-badge&logo=rust) ![Git](https://img.shields.io/badge/Git-Version%20Control-orange?style=for-the-badge&logo=git) |
| **项目状态** | ![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen?style=for-the-badge) ![Maintenance](https://img.shields.io/badge/Maintained-Yes-green?style=for-the-badge) |
| **版本信息** | ![Version](https://img.shields.io/badge/Version-0.1.0-blue?style=for-the-badge) ![Release](https://img.shields.io/badge/Release-Beta-yellow?style=for-the-badge) |

---

## ✨ 功能特性

- 🎯 **用户认证** - 注册、登录、JWT Token 认证
- 📝 **文章管理** - 完整的文章 CRUD 操作
- 🏷️ **标签与分类** - 灵活的分类和标签系统
- 💬 **评论系统** - 支持层级回复和审核功能
- 🔄 **版本控制** - 文章版本历史和回滚
- ⏰ **调度发布** - 定时发布文章
- 📊 **数据分析** - 文章访问统计和分析
- 🔍 **搜索功能** - 文章标题和内容搜索
- 🔗 **相关文章** - 基于分类和标签的推荐
- 📄 **草稿功能** - 自动保存和草稿管理
- 👥 **权限管理** - 基于角色的访问控制
- 🌐 **CORS 支持** - 跨域资源共享

---

## 🛠️ 技术栈

| 技术 | 版本 | 用途 |
|------|------|------|
| [Rust](https://www.rust-lang.org) | 1.75+ | 主要开发语言 |
| [Actix Web](https://actix.rs) | 4.7.0 | Web 框架 |
| [Diesel](https://diesel.rs) | 2.1.0 | ORM 和数据库操作 |
| [SQLite](https://www.sqlite.org) | 3.35+ | 数据库 |
| [jsonwebtoken](https://crates.io/crates/jsonwebtoken) | 9.3.0 | JWT 认证 |
| [bcrypt](https://crates.io/crates/bcrypt) | 0.15.0 | 密码加密 |
| [actix-cors](https://crates.io/crates/actix-cors) | 0.7.0 | 跨域处理 |
| [validator](https://crates.io/crates/validator) | 0.16.0 | 数据验证 |
| [chrono](https://crates.io/crates/chrono) | 0.4.38 | 时间处理 |
| [serde](https://crates.io/crates/serde) | 1.0.197 | 序列化/反序列化 |
| [tokio](https://tokio.rs) | 1.37.0 | 异步运行时 |

---

## 📂 项目结构

```
/workspace/
├── 📁 migrations/         # 数据库迁移文件
│   └── *.sql/            # 迁移脚本
├── 📁 src/               # 源代码
│   ├── main.rs           # 应用入口和路由配置
│   ├── models.rs         # 数据库模型定义
│   ├── schema.rs         # Diesel 数据库 schema
│   ├── handlers.rs       # API 处理函数
│   ├── auth.rs           # 认证相关功能
│   └── db.rs             # 数据库连接
├── 📄 .env               # 环境变量
├── 📄 Cargo.toml         # Cargo 项目配置
├── 📄 diesel.toml        # Diesel 配置
└── 📄 CODE_WIKI.md       # 详细项目文档
```

---

## 🚀 快速开始

### 环境要求

- 🦀 Rust 1.75 或更高版本
- 📦 Cargo 包管理器
- 🗄️ SQLite 3.35 或更高版本

### 安装步骤

1. **克隆项目**
   ```bash
   git clone <repository-url>
   cd workspace
   ```

2. **配置环境变量**
   
   项目已包含 [`.env`](file:///workspace/.env) 配置文件：
   ```
   DATABASE_URL=./blog.db
   ```

3. **运行数据库迁移**
   ```bash
   diesel migration run
   ```

4. **启动开发服务器**
   ```bash
   cargo run
   ```

5. **访问服务**
   
   服务将运行在：`http://localhost:8080`

### 生产构建

```bash
cargo build --release
```

---

## 📚 项目文档

详细的项目文档请查看 [CODE_WIKI.md](file:///workspace/CODE_WIKI.md)，包含：

- 📖 核心模块详细说明
- 🗄️ 完整的数据库设计文档
- 🔌 所有 API 接口文档
- 🔐 安全注意事项
- 💡 待实现功能列表

---

## 🔌 API 文档

### 认证接口

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| POST | `/api/auth/register` | 用户注册 | ❌ |
| POST | `/api/auth/login` | 用户登录 | ❌ |
| POST | `/api/auth/logout` | 用户登出 | ✅ |
| POST | `/api/auth/refresh` | 刷新 Token | ❌ |

### 用户接口

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/api/users/me` | 获取当前用户信息 | ✅ |
| PUT | `/api/users/me` | 更新当前用户资料 | ✅ |
| PUT | `/api/users/me/password` | 修改密码 | ✅ |

### 文章接口

| 方法 | 路径 | 描述 | 认证 |
|-----|-----|------|-----|
| GET | `/api/posts` | 获取文章列表（支持分页、筛选） | ❌ |
| GET | `/api/posts/{id}` | 获取单篇文章 | ❌ |
| POST | `/api/posts` | 创建文章 | ✅ |
| PUT | `/api/posts/{id}` | 更新文章 | ✅ |
| DELETE | `/api/posts/{id}` | 删除文章（软删除） | ✅ |

### 更多接口

完整的 API 文档请查看 [CODE_WIKI.md](file:///workspace/CODE_WIKI.md#-api-接口文档)。

---

## 🧪 使用示例

### 用户注册

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "demo",
    "email": "demo@example.com",
    "password": "password123"
  }'
```

### 用户登录

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "demo@example.com",
    "password": "password123"
  }'
```

### 获取文章列表

```bash
curl http://localhost:8080/api/posts?page=1&per_page=10
```

### 创建文章（需要认证）

```bash
curl -X POST http://localhost:8080/api/posts \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "我的第一篇博客",
    "content": "这是博客内容...",
    "status": "published"
  }'
```

---

## 🔐 安全说明

⚠️ **注意：** 这是一个开发版本，生产环境请务必进行以下改进：

1. 🔑 **JWT 密钥** - 更换强密钥并从环境变量读取
2. 🌐 **CORS** - 配置具体的白名单域名
3. 🗄️ **数据库** - 考虑使用 PostgreSQL 或 MySQL
4. 🔒 **HTTPS** - 使用 TLS 加密所有通信
5. ⏱️ **Token 过期** - 设置合理的过期时间

更多安全细节请查看 [CODE_WIKI.md](file:///workspace/CODE_WIKI.md#-安全注意事项)。

---

## 🤝 贡献指南

我们欢迎所有形式的贡献！

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 许可证

本项目采用 MIT 许可证发布。查看 [LICENSE](file:///workspace/LICENSE) 文件了解更多信息。

---

## 👥 社区支持

- 📖 [项目 Wiki](file:///workspace/CODE_WIKI.md)
- 📝 [问题反馈](https://github.com/your-username/blog-api/issues)
- 💬 [讨论区](https://github.com/your-username/blog-api/discussions)

---

## 🙏 致谢

感谢 Rust 社区和 Actix-web 团队提供的优秀框架！

---

<div align="center">
  <sub>由 ❤️ 驱动</sub>
</div>
