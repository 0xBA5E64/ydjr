-- Add up migration script here
CREATE VIEW video_metadata_view AS SELECT
    metadata->>'fulltitle' title,
    metadata->>'duration' duration,
    metadata->>'channel' channel,
    metadata->>'id' AS id,
    metadata->>'upload_date' AS upload_date,
    metadata->>'view_count' AS views,
    video_path
FROM videos ORDER BY concat(video_path, title) DESC;
