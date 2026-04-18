use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sqlx::{Pool, Sqlite};
use std::path::PathBuf;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Failed to open file")]
    FileOpenError,
    #[error("Failed to parse file")]
    FileParseError,
    #[error("Failed to find embedded json in file")]
    FindJsonEmbedError,
    #[error("Failed to parse json")]
    JsonParseError,
}

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Error during metadata extraction: {0}")]
    MetadataExtractionError(ExtractError),
    #[error("Failed to perform database insert")]
    DatabaseError,
    #[error("No videos found")]
    NoVideos,
}

fn extract_json_metadata(file: &PathBuf) -> Result<serde_json::Value, ExtractError> {
    // Parse the Matroska file
    let matroska = matroska::open(file).map_err(|_| ExtractError::FileParseError)?;
    // Find the json attachment
    let json_attachment: matroska::Attachment = matroska
        .attachments
        .into_iter()
        .find(|x| x.name.ends_with(".json"))
        .ok_or(ExtractError::FindJsonEmbedError)?;

    // Parse it as JSON and return the result
    serde_json::from_slice(&json_attachment.data).map_err(|_| ExtractError::JsonParseError)
}

pub async fn index_video(path: &PathBuf, db_pool: &Pool<Sqlite>) -> Result<(), IndexError> {
    let video_path: String = path.to_string_lossy().to_string();

    let json: serde_json::Value =
        extract_json_metadata(path).map_err(IndexError::MetadataExtractionError)?;
    sqlx::query!(
        "INSERT INTO videos (video_path, metadata) VALUES (?1, jsonb(?2)) ON CONFLICT (video_path) DO UPDATE SET metadata=excluded.metadata",
        video_path,
        json
    )
    .execute(db_pool)
    .await
    .map_err(|_| IndexError::DatabaseError)?;
    Ok(())
}

pub async fn index_videos_recursively(
    in_dir: PathBuf,
    db_pool: &Pool<Sqlite>,
    headless_mode: bool,
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

    if files.is_empty() {
        return Err(IndexError::NoVideos);
    }
    log::info!("Found {} videos to index.", files.len());

    let bar = multi_progress.add(ProgressBar::new(files.len().try_into().unwrap()));

    if !headless_mode {
        bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise} / {duration_precise}] {wide_bar} [{human_pos}/{human_len}]",
            )
            .unwrap(),
        );
    }

    for path in files {
        if let Err(error) = index_video(&path, db_pool).await {
            log::error!("Couldn't index \"{}\" - {}", path.to_string_lossy(), error);
            continue;
        };

        if !headless_mode {
            bar.inc(1);
        }
    }

    if !headless_mode {
        bar.finish();
    }
    Ok(())
}
