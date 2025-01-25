-- Migration number: 0001 	 2025-01-15T23:14:42.890Z

CREATE TABLE IF NOT EXISTS users (
    id       integer NOT NULL PRIMARY KEY,
    token    text    NOT NULL, -- just for demo
    name     text    NOT NULL,
    location text,
    age      integer,

    UNIQUE (name, token)
);

CREATE TABLE IF NOT EXISTS tweets (
    id        integer NOT NULL PRIMARY KEY,
    user_id   integer NOT NULL,
    content   text    NOT NULL,
    posted_at text    NOT NULL  -- unix timestamp as text
);
