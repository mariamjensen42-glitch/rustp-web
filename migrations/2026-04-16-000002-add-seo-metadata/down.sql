ALTER TABLE posts DROP COLUMN IF EXISTS seo_title;
ALTER TABLE posts DROP COLUMN IF EXISTS seo_description;
ALTER TABLE posts DROP COLUMN IF EXISTS seo_keywords;
ALTER TABLE posts DROP COLUMN IF EXISTS seo_canonical;
ALTER TABLE posts DROP COLUMN IF EXISTS seo_robots;

-- Also remove SEO fields from post_versions table
ALTER TABLE post_versions DROP COLUMN IF EXISTS seo_title;
ALTER TABLE post_versions DROP COLUMN IF EXISTS seo_description;
ALTER TABLE post_versions DROP COLUMN IF EXISTS seo_keywords;
ALTER TABLE post_versions DROP COLUMN IF EXISTS seo_canonical;
ALTER TABLE post_versions DROP COLUMN IF EXISTS seo_robots;