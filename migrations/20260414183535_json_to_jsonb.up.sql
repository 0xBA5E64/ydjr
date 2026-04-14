-- Add up migration script here
CREATE TABLE videos_new (
    video_path text NOT NULL UNIQUE,
    metadata blob NOT NULL
);

INSERT INTO videos_new (video_path, metadata)
SELECT video_path, jsonb(metadata) FROM videos;

DROP TABLE videos;
ALTER TABLE videos_new RENAME TO videos;
