CREATE TABLE post_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    slug TEXT,
    content TEXT NOT NULL,
    excerpt TEXT,
    author TEXT NOT NULL,
    status TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    published_at TIMESTAMP,
    category_id INTEGER,
    user_id INTEGER,
    summary TEXT,
    cover_image TEXT,
    is_published BOOLEAN,
    is_top BOOLEAN,
    allow_comments BOOLEAN,
    version_number INTEGER NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);

CREATE INDEX idx_post_versions_post_id ON post_versions(post_id);
CREATE INDEX idx_post_versions_version_number ON post_versions(post_id, version_number);