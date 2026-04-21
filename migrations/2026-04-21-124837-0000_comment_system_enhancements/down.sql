-- 删除 comment_likes 表
DROP TABLE IF EXISTS comment_likes;

-- 删除 comment_notifications 表
DROP TABLE IF EXISTS comment_notifications;

-- 修改 comments 表，删除新增的列
ALTER TABLE comments DROP COLUMN likes_count;
ALTER TABLE comments DROP COLUMN sort_order;
ALTER TABLE comments DROP COLUMN notification_sent;