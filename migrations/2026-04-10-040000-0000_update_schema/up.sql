-- 修改 posts 表
ALTER TABLE posts ADD COLUMN slug TEXT;
ALTER TABLE posts ADD COLUMN excerpt TEXT;
ALTER TABLE posts ADD COLUMN status TEXT DEFAULT 'draft';
ALTER TABLE posts ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE posts ADD COLUMN published_at TIMESTAMP;

-- 修改 categories 表，先添加没有 UNIQUE 约束的列
ALTER TABLE categories ADD COLUMN slug TEXT;
ALTER TABLE categories ADD COLUMN description TEXT;

-- 修改 tags 表，先添加没有 UNIQUE 约束的列
ALTER TABLE tags ADD COLUMN slug TEXT;

-- 创建 media 表
CREATE TABLE IF NOT EXISTS media (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL,
    mimetype TEXT NOT NULL,
    size INTEGER NOT NULL,
    uploaded_by INTEGER REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 为现有数据填充默认的 slug 值
-- 注意：这只是一个简单的处理，实际生产环境中可能需要更复杂的逻辑
UPDATE categories SET slug = name WHERE slug IS NULL;
UPDATE tags SET slug = name WHERE slug IS NULL;