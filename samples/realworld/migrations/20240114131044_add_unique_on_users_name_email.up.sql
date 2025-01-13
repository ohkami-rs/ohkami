-- Add up migration script here
ALTER TABLE users ADD CONSTRAINT users_name_is_unique UNIQUE (name);
ALTER TABLE users ADD CONSTRAINT users_email_is_unique UNIQUE (email);