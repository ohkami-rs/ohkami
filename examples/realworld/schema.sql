-- core tables

CREATE TABLE IF NOT EXISTS articles (
    id          uuid         NOT NULL DEFAULT gen_random_uuid(),
    author_id   uuid         NOT NULL,
    slug        
    title       varchar(64)  NOT NULL,
    description varchar(256),
    body        text         NOT NULL,
    created_at  timestampz   NOT NULL DEFAULT now(),
    updated_at  timestampz   NOT NULL DEFAULT now(),

    PRIMARY KEY (id),
    FOREIGN KEY (author_id) REFERENCES users (id)    
);

CREATE TABLE IF NOT EXISTS users (
    id        uuid         NOT NULL DEFAULT gen_random_uuid(),
    email     varchar(32)  NOT NULL,
    name      varchar(32)  NOT NULL,
    bio       varchar(256),
    image_url varchar(64),

    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS comments (
    id         uuid NOT NULL DEFAULT gen_random_uuid(),
    author_id  uuid NOT NULL,
    article_id uuid NOT NULL,
    content    text NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (author_id)  REFERENCES users (id),
    FOREIGN KEY (article_id) REFERENCES articles (id)
);

CREATE TABLE IF NOT EXISTS tags (
    id   uuid        NOT NULL DEFAULT gen_random_uuid(),
    name varchar(32) NOT NULL,

    PRIMARY KEY (id)
);


-- intermediate tables

CREATE TABLE IF NOT EXISTS articles_tags (
    id         uuid NOT NULL DEFAULT gen_random_uuid(),
    tag_id     uuid NOT NULL,
    article_id uuid NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (tag_id)     REFERENCES tags (id),
    FOREIGN KEY (article_id) REFERENCES articles (id)
);

CREATE TABLE IF NOT EXISTS users_favorite_articles (
    id         uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id    uuid NOT NULL,
    article_id uuid NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (user_id)    REFERENCES users (id),
    FOREIGN KEY (article_id) REFERENCES articles (id)
);

CREATE TABLE IF NOT EXISTS users_follow_users (
    id          uuid NOT NULL DEFAULT gen_random_uuid(),
    follower_id uuid NOT NULL,
    followee_id uuid NOT NULL,

    PRIMARY KEY (id),
    FOREIGN KEY (follower_id) REFERENCES users (id),
    FOREIGN KEY (followee_id) REFERENCES users (id)
);
