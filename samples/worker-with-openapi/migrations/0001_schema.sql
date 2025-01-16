-- Migration number: 0001 	 2025-01-15T23:14:42.890Z

CREATE TABLE IF NOT EXISTS users (
    id      integer     NOT NULL PRIMARY KEY,
    token   varchar(64) NOT NULL, -- just for demo
    name    varchar(64) NOT NULL,
    country varchar(64),
    age     integer,

    UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS tweets (
    id        integer NOT NULL PRIMARY KEY,
    user_id   integer NOT NULL,
    content   text    NOT NULL,
    posted_at integer NOT NULL, -- unix timestamp

    FOREIGN KEY user_id REFERENCES users (id)
);
