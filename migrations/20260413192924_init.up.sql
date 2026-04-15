-- Add up migration script here
CREATE TABLE videos (
    video_path text NOT NULL UNIQUE,
    metadata text NOT NULL
)
