-- Add up migration script here
ALTER TABLE comments DROP COLUMN id;
ALTER TABLE comments ADD  COLUMN id serial NOT NULL;
ALTER TABLE comments ADD PRIMARY KEY (id);