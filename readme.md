`ydjr` is a utility to ingest embedded json metadata included in [yt-dlp](https://github.com/yt-dlp/yt-dlp) videos downloaded with [`--embed-info-json`](https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#post-processing-options) to a SQLite database for further querying. Each row of video appears to roughtly equate to 360 KiB.

## why "ydjr"?

This codebase started off as a project to rename youtube downloads made with yt-dlp based on their embedded json metadata to help better manage large long-term collections of videos that may have inconsistent naming schemes, hence **yt**-dlp **j**son **r**enamer

However, as time went on, I realized the power of having this data easily accessible in the form of a searchable database. From there, multiple tools could be developed, such as a renamer, a local search engine that could utilize more metadata, and a file integrity validator of sorts, in the form of this indexer.

All in all, this SQLite database can act as a universal API to be used from any language and platform to navigate one's local online video collection.

## Example queries with SQLite
Since SQLite supports [jsonb](https://sqlite.org/json1.html#jminib) data since 3.45.0 (2024-01-15) and a more convenient [operator-based json extraction syntax](https://sqlite.org/json1.html#jptr) since 3.38.0 (2022-02-22), extracting potential data from this database is trivial. To show the 32 longest uploads in your collection you can for instance query this:
```sql
SELECT
    video_path File,
    metadata->>'title' Title,
    metadata->>'duration_string' Duration,
    metadata->>'channel' Channel
FROM videos
    ORDER BY
        metadata->>'duration' DESC
    LIMIT 32;
```
