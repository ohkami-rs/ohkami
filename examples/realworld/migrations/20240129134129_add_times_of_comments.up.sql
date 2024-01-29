-- Add up migration script here
ALTER TABLE comments ADD COLUMN created_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE comments ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();