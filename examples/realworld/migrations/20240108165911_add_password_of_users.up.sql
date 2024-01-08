-- Add up migration script here
ALTER TABLE users ADD COLUMN password varchar(64) NOT NULL;