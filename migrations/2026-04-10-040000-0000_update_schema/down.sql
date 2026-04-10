ALTER TABLE posts DROP COLUMN slug;
ALTER TABLE posts DROP COLUMN excerpt;
ALTER TABLE posts DROP COLUMN status;
ALTER TABLE posts DROP COLUMN deleted_at;
ALTER TABLE posts DROP COLUMN published_at;

ALTER TABLE categories DROP COLUMN slug;
ALTER TABLE categories DROP COLUMN description;

ALTER TABLE tags DROP COLUMN slug;

DROP TABLE media;
