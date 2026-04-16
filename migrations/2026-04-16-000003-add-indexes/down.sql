-- 移除 posts 表的索引
DROP INDEX IF EXISTS idx_posts_user_id;
DROP INDEX IF EXISTS idx_posts_is_published;
DROP INDEX IF EXISTS idx_posts_is_top;

-- 移除 categories 表的索引
DROP INDEX IF EXISTS idx_categories_parent_id;

-- 移除 tags 表的索引
DROP INDEX IF EXISTS idx_tags_name;