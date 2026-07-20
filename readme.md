`ydjr` is a utility to ingest videos downloaded with [youtube-dl](https://github.com/ytdl-org/youtube-dl) / [yt-dlp](https://github.com/yt-dlp/yt-dlp) with embedded json metadata added with [`--embed-info-json`](https://github.com/yt-dlp/yt-dlp/blob/2c28ee5d76d2c0d350407fd81dbdd71394b67993/README.md?plain=1#L1015-L1016) to a SQLite database for further querying. Each row of video appears to roughtly equate to 360 KiB with the current database schema.

## why "ydjr"?
This codebase started off as a project meant to rename YouTube downloads made through yt-dlp / youtube-dl based on their JSON metadata "attachments" to help better manage large long-term collections of downloaded videos that may have inconsistent naming schemes, hence **yt**-dlp **j**son **r**enamer

However, as time went on, I've realized the power of having this data easily accessible in the form of a universally queryable database. From there, multiple tools could be developed, such as a renamer, a local search engine, and a file integrity validator of sorts, in the form of this indexer.

All in all, this SQLite database can go on to act as a universal API of sorts to be used from any language and platform to navigate one's local online video collection.

> [!CAUTION]
> The database that ydjr produces is mainly intended for private usage, where all who may access it are to be trusted. Although [post-2021 versions of yt-dlp may make an effort to clean the infojson of sensitive personal information](https://github.com/yt-dlp/yt-dlp/commit/e4f0275711cd4917bfe819356533926cd369a621), it is worth considering that there may still be personally identifiable information such as usernames in file paths (also saved during mediainfo indexing), system info, and other private information from sessions authenticated with cookies for streams that required credentials, etc., which you may still wish to keep private. 

## good to know
- ydjr tries to keep external dependencies to a minimum, but at present requires `libmediainfo.so.0` for mediainfo indexing, plus the enviroument variable `LC_ALL` set to `C.UTF-8` for it to support indexing files with non-ascii filenames. As such, both of these are managed by default by the containerized builds for those whom desire a portable solution.

- ydjr is meant to be rerun on the same collection over time to update the database. Along with `--remove-missing` / `-r`, ydjr will cross-reference the videos in the directory you point it to with what it can already find in its current database and only insert (and remove, if specified) entries as needed. ydjr will also automatically perform database migrations when invoked on a pre-existing database to update it to the current schema.

- ydjr (un-)intentionally does not make an opinionated stance about relative versus absolute paths. Whichever way you define the index path parameter is how the `video_path`'s will be saved in your database:
  - **To maintain portability** for your database Consider invoking ydjr from the root of your collection.
    ```bash
    cd /mnt/yt-backups/
    ydjr index .
    ```
  - **If your collection is spread out across your system** you may instead wish to set fixed location for your database and index with full, absolute paths instead.
    ```bash
    ydjr --db ~/.ydjr.db index /home/null/Downloads/YouTube/
    # or since this might be a tad cumbersome, consider a local alias:
    alias yi=ydjr --db ~/.ydjr.db index
    yi /home/null/Videos/
    yi /mnt/ext-disk/
    ```
- ydjr can be used via Docker. This may be useful on a NAS-like systems where a job such as this might get automatically called via a cron-like service.
  ```bash
  docker run --rm -v /path/to/videos:/mnt ghcr.io/0xBA5E64/ydjr ydjr index -hr .
  ```
  -replacing `/path/to/videos` with wherever you have videos to index. (`-h` for headless, `-r` to remove videos no longer present on the file-system from the database)

  - If you are looking to index videos stored on remote storage (eg; a SMB Share) you must explicitly specify the location of the database somewhere local:
    ```bash
    cd /mnt/my-nas/youtube-videos/
    ydjr --db ~/ydjr.db index .
    ```
    This is due to limitations in sqlite and it's ability to lock the database. See [this page](https://sqlite.org/useovernet.html) on sqlite.org for more information I frankly don't fully grasp.

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

The `video_metadata_view` view can also be queried for simpler overviews:

```sql
SELECT * FROM video_metadata_view
    ORDER BY duration DESC
    LIMIT 32;
```

### developer notes
To work with this codebase you'll want the `sqlx-cli`, and to initialize a database for the sake of its query checker.
```bash
cargo install sqlx-cli
sqlx database setup -D sqlite:dev.sqlite
```
You may need to restart your rust-analyze server after this

## Reasoning for design decisions

### Why not deconstruct the json metadata into database columns?
The youtube-dl / yt-dlp json metadata schema does not appear to be stable, and as such, **all data could not reliably be serialized into standardized columns**. Different versions of these tools may also have generated and attached different data. Since SQLite makes it trivial to work with json, and in an effort to preserve as much of this data as possible, the whole of the json data is parsed, validated, and inserted into the database.

However, for convenience, the `video_metadata_view` [table view](https://sqlite.org/lang_createview.html) may also be used for ease of querying.

### Why SQLite? Why not a Key-value store database? or a document database?
With the current schema consisting of only three colums: `video_path`, `metadata` & `mediainfo`, one might argue this job would be a better fit for a Key-value database, like Redis or one of it's many open-source derivatives, or a document database such as MongoDB to better fit the unstructured nature of the json metadata.

However, SQLite is somethat more standardized, and this approach also allows for further expansion into things like adding a mediainfo dump to each colum.

Also, this project is perhaps first and foremost an exercise in SQL for myself, and a fun way to practically begin using a real SQLite database, filled with thousands of rows of real, actual data.

## print-json
Since retrieving the embedded json data itself isn't typically trivial, ydjr now also comes with a simple helper function called  `print-json` to just print the embedded json of any file. This can then be used in other scripts as well. You may however also wish strip some of the more noisy fields using something like `jq` for easier reading like so:
```bash
$ ydjr print-json "video.mkv"|jq 'del(.["thumbnails","formats","automatic_captions"])' 
```
