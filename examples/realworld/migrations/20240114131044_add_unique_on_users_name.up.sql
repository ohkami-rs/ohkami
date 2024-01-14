-- Add up migration script here
ALTER TABLE users ADD CONSTRAINT users_names_are_unique UNIQUE (name);