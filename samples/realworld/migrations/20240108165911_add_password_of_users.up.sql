-- Add up migration script here
ALTER TABLE users ADD COLUMN password varchar(128) NOT NULL;
ALTER TABLE users ADD COLUMN salt varchar(128) NOT NULL;