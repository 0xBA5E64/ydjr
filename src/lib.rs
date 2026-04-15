use sqlx::{Pool, Sqlite};
use std::{io, path::PathBuf};
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

pub async fn index_videos(in_dir: PathBuf, db_pool: &Pool<Sqlite>) -> io::Result<()> {
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
    println!("Found {} videos.", files.len());
    for path in files {
        let video_path: String = path.to_string_lossy().to_string();

        println!("Processing video: {}", video_path);

        let json: serde_json::Value = match extract_json_metadata(&path) {
            Ok(value) => value,
            Err(error) => {
                println!(
                    "Error, couldn't index \"{}\" - {}",
                    path.to_string_lossy(),
                    error
                );
                continue;
            }
        };

        sqlx::query!(
            "INSERT INTO videos (video_path, metadata) VALUES (?1, jsonb(?2))",
            video_path,
            json
        )
        .execute(db_pool)
        .await
        .expect("Unable to commit to database");
    }

    Ok(())
}
