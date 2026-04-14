-- Add down migration script here
CREATE TABLE videos_old (
    video_path text NOT NULL UNIQUE,
    metadata text NOT NULL
);

INSERT INTO videos_old (video_path, metadata)
SELECT video_path, json(metadata) FROM videos;

DROP TABLE videos;
ALTER TABLE videos_old RENAME TO videos;
