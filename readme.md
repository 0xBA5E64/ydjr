`ydjr` is a utility to ingest videos downloaded with [youtube-dl](https://github.com/ytdl-org/youtube-dl) / [yt-dlp](https://github.com/yt-dlp/yt-dlp) with embedded json metadata added with [`--embed-info-json`](https://github.com/yt-dlp/yt-dlp/blob/2c28ee5d76d2c0d350407fd81dbdd71394b67993/README.md?plain=1#L1015-L1016) to a SQLite database for further querying. Each row of video appears to roughtly equate to 360 KiB with the current database schema.

## why "ydjr"?
This codebase started off as a project meant to rename youtube downloads made through yt-dlp / youtube-dl based on their json metadata "attachments" to help better manage large long-term collections of downloaded videos that may have inconsistent naming schemes, hence **yt**-dlp **j**son **r**enamer

However, as time went on, I've realized the power of having this data easily accessible in the form of a universally queryable database. From there, multiple tools could be developed, such as a renamer, a local search engine, and a file integrity validator of sorts, in the form of this indexer.

All in all, this SQLite database can go on to act as a universal API of sorts to be used from any language and platform to navigate one's local online video collection.

## Example queries with SQLite
Since SQLite supports [jsonb](https://sqlite.org/json1.html#jminib) data since 3.45.0 (2024-01-15) and a more convenient [operator-based json extraction syntax](https://sqlite.org/json1.html#jptr) since 3.38.0 (2022-02-22), extracting potential data from this database is trivial. To show the 32 longest uploads in your index you can for instance query this:
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
