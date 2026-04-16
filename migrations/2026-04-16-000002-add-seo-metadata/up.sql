ALTER TABLE posts ADD COLUMN seo_title TEXT DEFAULT NULL;
ALTER TABLE posts ADD COLUMN seo_description TEXT DEFAULT NULL;
ALTER TABLE posts ADD COLUMN seo_keywords TEXT DEFAULT NULL;
ALTER TABLE posts ADD COLUMN seo_canonical TEXT DEFAULT NULL;
ALTER TABLE posts ADD COLUMN seo_robots TEXT DEFAULT NULL;

-- Also add SEO fields to post_versions table for consistency
ALTER TABLE post_versions ADD COLUMN seo_title TEXT DEFAULT NULL;
ALTER TABLE post_versions ADD COLUMN seo_description TEXT DEFAULT NULL;
ALTER TABLE post_versions ADD COLUMN seo_keywords TEXT DEFAULT NULL;
ALTER TABLE post_versions ADD COLUMN seo_canonical TEXT DEFAULT NULL;
ALTER TABLE post_versions ADD COLUMN seo_robots TEXT DEFAULT NULL;