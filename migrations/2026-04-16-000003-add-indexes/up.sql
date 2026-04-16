-- 为 posts 表添加索引
CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_is_published ON posts(is_published);
CREATE INDEX IF NOT EXISTS idx_posts_is_top ON posts(is_top);

-- 为 categories 表添加索引
CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);

-- 为 tags 表添加索引
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);