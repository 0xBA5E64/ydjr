-- Add up migration script here
CREATE TABLE failed_videos (
    video_path text NOT NULL UNIQUE,
    error text NOT NULL
);
