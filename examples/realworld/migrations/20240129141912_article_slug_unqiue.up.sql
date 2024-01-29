-- Add up migration script here
ALTER TABLE articles ADD CONSTRAINT article_should_have_unique_slug UNIQUE (slug);