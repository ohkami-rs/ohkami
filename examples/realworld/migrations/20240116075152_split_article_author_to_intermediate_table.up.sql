-- Add up migration script here
ALTER TABLE articles DROP COLUMN author_id;

CREATE TABLE IF NOT EXISTS users_author_of_articles (
    id         uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id    uuid NOT NULL,
    article_id uuid NOT NULL,

    FOREIGN KEY (user_id)    REFERENCES users (id),
    FOREIGN KEY (article_id) REFERENCES articles (id),
    CONSTRAINT author_is_unique UNIQUE (user_id, article_id)
);