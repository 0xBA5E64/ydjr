use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use matroska::MatroskaError;
use rsmediainfo::{MediaInfo, MediaInfoError};
use sqlx::{Pool, Sqlite};
use std::{path::PathBuf, str::FromStr};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Failed to open Matroska file: {0}")]
    MatroskaOpenError(MatroskaError),
    #[error("Failed to find json attachment")]
    FindAttachedJsonError,
    #[error("Failed to parse json: {0}")]
    JsonParseError(serde_json::Error),
}

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Json Extraction Error: {0}")]
    MetadataExtractionError(ExtractError),
    #[error("Mediainfo generation Error: {0}")]
    MediainfoGenerationError(MediaInfoError),
    #[error("Failed to perform database insert")]
    DatabaseError,
    #[error("No videos found")]
    NoVideos,
}

fn extract_json_metadata(file: &PathBuf) -> Result<serde_json::Value, ExtractError> {
    // Parse the Matroska file
    let matroska = matroska::open(file).map_err(ExtractError::MatroskaOpenError)?;
    // Find the json attachment
    let json_attachment: matroska::Attachment = matroska
        .attachments
        .into_iter()
        .find(|x| x.name.ends_with(".json"))
        .ok_or(ExtractError::FindAttachedJsonError)?;

    // Parse it as JSON and return the result
    serde_json::from_slice(&json_attachment.data).map_err(ExtractError::JsonParseError)
}

fn get_video_mediainfo(file: &PathBuf) -> Result<serde_json::Value, IndexError> {
    Ok(serde_json::Value::from(
        MediaInfo::parse_media_info_path(file)
            .map_err(IndexError::MediainfoGenerationError)?
            .to_data(),
    ))
}

pub async fn index_video(path: &PathBuf, db_pool: &Pool<Sqlite>) -> Result<(), IndexError> {
    let video_path: String = path.to_string_lossy().to_string();

    let json: serde_json::Value =
        extract_json_metadata(path).map_err(IndexError::MetadataExtractionError)?;
    let mediainfo: serde_json::Value =
        get_video_mediainfo(path).map_err(|_| IndexError::NoVideos)?;

    sqlx::query!(
        "INSERT INTO videos (video_path, metadata, mediainfo) VALUES (?1, jsonb(?2), jsonb(?3)) ON CONFLICT (video_path) DO UPDATE SET metadata=excluded.metadata",
        video_path,
        json,
        mediainfo
    )
    .execute(db_pool)
    .await
    .map_err(|_| IndexError::DatabaseError)?;
    Ok(())
}

pub async fn index_videos_recursively(
    in_dir: PathBuf,
    db_pool: &Pool<Sqlite>,
    remove_missing: bool,
    headless: bool,
    multi_progress: MultiProgress,
) -> Result<(), IndexError> {
    let files: Vec<PathBuf> = tokio::task::block_in_place(|| {
        WalkDir::new(&in_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("mkv"))
            })
            .map(|x| x.path().to_path_buf())
            .collect()
    });

    log::info!("Found {} videos to index.", files.len());

    if files.is_empty() {
        return Err(IndexError::NoVideos);
    }

    // Get Vec<PathBuf> of all videos already indexed
    let videos_indexed: Vec<PathBuf> = sqlx::query!("SELECT video_path FROM videos;")
        .fetch_all(db_pool)
        .await
        .unwrap()
        .iter()
        .map(|i| PathBuf::from_str(&i.video_path).unwrap())
        .collect();

    if remove_missing {
        for file in &videos_indexed {
            let path_str = file.to_string_lossy().to_string();
            let file_exists = file.try_exists().unwrap_or(false);

            if !file_exists {
                log::error!("Couldn't find \"{}\" - Removing from Database", path_str);
                sqlx::query!("DELETE FROM failed_videos WHERE video_path = ?1", path_str)
                    .execute(db_pool)
                    .await
                    .map_err(|_| IndexError::DatabaseError)?;
                continue;
            }
        }
    }

    log::info!("DB has {} videos", videos_indexed.len());

    // Filter files form files already found in videos_indexed
    let files: Vec<&PathBuf> = files
        .iter()
        .filter(|x| !videos_indexed.contains(x))
        .collect();

    log::info!("File-list reduced to {} by comparing to DB.", files.len());

    let bar = multi_progress.add(ProgressBar::new(files.len().try_into().unwrap()));

    if !headless {
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise} / {duration_precise}] {wide_bar} [{human_pos}/{human_len}]",
            )
            .unwrap(),
        );
    }

    for path in files {
        let path_str = path.to_string_lossy().to_string();

        if let Err(error) = index_video(path, db_pool).await {
            let err_str = error.to_string();

            log::error!("Couldn't index \"{}\" - {}", path_str, err_str);

            sqlx::query!(
                "INSERT INTO failed_videos (video_path, error) VALUES (?1, ?2) ON CONFLICT (video_path) DO UPDATE SET error=excluded.error",
                path_str,
                err_str
            )
            .execute(db_pool)
            .await
            .map_err(|_| IndexError::DatabaseError)?;

            continue;
        };

        if !headless {
            bar.inc(1);
        }
    }

    if !headless {
        bar.finish();
    }
    Ok(())
}

pub async fn reindex_failed_videos(
    db_pool: &Pool<Sqlite>,
    remove_missing: bool,
    headless: bool,
    multi_progress: MultiProgress,
) -> Result<(), IndexError> {
    // Get Vec<PathBuf> of all videos already indexed
    let previous_failed: Vec<PathBuf> = sqlx::query!("SELECT video_path FROM failed_videos;")
        .fetch_all(db_pool)
        .await
        .unwrap()
        .iter()
        .map(|i| PathBuf::from_str(&i.video_path).unwrap())
        .collect();

    if previous_failed.is_empty() {
        return Err(IndexError::NoVideos);
    }

    log::info!("Found {} previously failed videos.", previous_failed.len());

    let bar = multi_progress.add(ProgressBar::new(previous_failed.len().try_into().unwrap()));

    if !headless {
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise} / {duration_precise}] {wide_bar} [{human_pos}/{human_len}]",
            )
            .unwrap(),
        );
    }
    for path in previous_failed {
        let path_str = path.to_string_lossy().to_string();
        let file_exists = path.try_exists().unwrap_or(false);

        if remove_missing && !file_exists {
            log::error!("Couldn't find \"{}\" - Removing from Database", path_str);
            sqlx::query!("DELETE FROM failed_videos WHERE video_path = ?1", path_str)
                .execute(db_pool)
                .await
                .map_err(|_| IndexError::DatabaseError)?;
            continue;
        }

        if let Err(error) = index_video(&path, db_pool).await {
            let err_str = error.to_string();

            log::error!("Couldn't index \"{}\" - {}", path_str, err_str);

            sqlx::query!(
                "INSERT INTO failed_videos (video_path, error) VALUES (?1, ?2) ON CONFLICT (video_path) DO UPDATE SET error=excluded.error",
                path_str,
                err_str
            )
            .execute(db_pool)
            .await
            .map_err(|_| IndexError::DatabaseError)?;
            continue;
        };

        if !headless {
            bar.inc(1);
        }
    }

    if !headless {
        bar.finish();
    }

    Ok(())
}
