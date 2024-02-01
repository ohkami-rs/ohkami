-- Add up migration script here

-- core tables

CREATE TABLE IF NOT EXISTS users (
    id        uuid         NOT NULL DEFAULT gen_random_uuid(),
    email     varchar(32)  NOT NULL,
    name      varchar(32)  NOT NULL,
    bio       varchar(512),
    image_url varchar(64),

    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS articles (
    id          uuid         NOT NULL DEFAULT gen_random_uuid(),
    author_id   uuid         NOT NULL,
    slug        varchar(128) NOT NULL,
    title       varchar(128) NOT NULL,
    description varchar(512) NOT NULL,
    body        text         NOT NULL,
    created_at  timestamptz  NOT NULL DEFAULT now(),
    updated_at  timestamptz  NOT NULL DEFAULT now(),

    PRIMARY KEY (id),
    UNIQUE (slug),
    FOREIGN KEY (author_id) REFERENCES users (id)    
);

CREATE TABLE IF NOT EXISTS comments (
    id         int         NOT NULL,
    article_id uuid        NOT NULL,
    author_id  uuid        NOT NULL,
    content    text        NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),

    PRIMARY KEY (id, article_id),
    FOREIGN KEY (article_id) REFERENCES articles (id),
    FOREIGN KEY (author_id)  REFERENCES users (id)
);

CREATE TABLE IF NOT EXISTS tags (
    id   serial      NOT NULL,
    name varchar(32) NOT NULL,

    PRIMARY KEY (id),
    UNIQUE(name)
);

-- intermediate tables

CREATE TABLE IF NOT EXISTS articles_have_tags (
    id         uuid NOT NULL DEFAULT gen_random_uuid(),
    article_id uuid NOT NULL,
    tag_id     int  NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (article_id) REFERENCES articles (id),
    FOREIGN KEY (tag_id)     REFERENCES tags (id)
);

CREATE TABLE IF NOT EXISTS users_follow_users (
    id          uuid NOT NULL DEFAULT gen_random_uuid(),
    follower_id uuid NOT NULL,
    followee_id uuid NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (follower_id) REFERENCES users (id),
    FOREIGN KEY (followee_id) REFERENCES users (id)
);

CREATE TABLE IF NOT EXISTS users_favorite_articles (
    id         uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id    uuid NOT NULL,
    article_id uuid NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (user_id)    REFERENCES users (id),
    FOREIGN KEY (article_id) REFERENCES articles (id)
);
