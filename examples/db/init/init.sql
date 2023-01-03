CREATE TABLE users (
  id         BIGSERIAL NOT NULL PRIMARY KEY,
  name       VARCHAR(255),
  created_at TIMESTAMP NOT NULL default CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL default CURRENT_TIMESTAMP
);

CREATE TABLE posts (
  id         BIGSERIAL    NOT NULL PRIMARY KEY,
  user_id    BIGINT       NOT NULL,
  title      VARCHAR(255) NOT NULL,
  body       TEXT,
  created_at TIMESTAMP NOT NULL default CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL default CURRENT_TIMESTAMP,
  CONSTRAINT fk_posts_user_id FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX posts_user_id ON posts (user_id);