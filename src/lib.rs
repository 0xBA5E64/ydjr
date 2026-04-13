use sqlx::{Acquire, SqliteConnection};
use std::fs::DirEntry;
use std::{fs, io, path::PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Failed to open file")]
    FileOpenError,
    #[error("Failed to parse file")]
    FileParseError,
    #[error("Failed to find json in file")]
    FindJsonEmbedError,
    #[error("Failed to parse json")]
    JsonParseError,
}

fn extract_json_metadata(file: &PathBuf) -> Result<serde_json::Value, ExtractError> {
    // Parse the Matroska file
    let matroska = matroska::open(file).map_err(|_| ExtractError::FileParseError)?;
    // Find the json attachment
    let json_attachment = matroska
        .attachments
        .into_iter()
        .find(|x| x.name.ends_with(".json"))
        .ok_or(ExtractError::FindJsonEmbedError)?;

    // Parse it as JSON and return the result
    serde_json::from_slice(&json_attachment.data).map_err(|_| ExtractError::JsonParseError)
}

// Renames "file" to "new_name" without changing its directory
fn rename_file(file: &PathBuf, new_name: &str) -> io::Result<()> {
    let mut new_path = file.clone();
    new_path.set_file_name(new_name);
    fs::rename(file, new_path)
}

pub fn rename_video(file: &PathBuf) -> io::Result<()> {
    // Extract JSON metadata from video-file
    let json: serde_json::Value = extract_json_metadata(file).unwrap();

    // Construct new filename
    let new_filename = format!(
        "{} [{}].mkv",
        json["title"].as_str().unwrap(),
        json["id"].as_str().unwrap()
    );

    // Finally, rename the file to the right filename.
    rename_file(file, &new_filename)
}

pub fn rename_videos(in_dir: PathBuf) -> io::Result<()> {
    let mut renamed_videos: Vec<PathBuf> = Vec::new();
    let mut failed_videos: Vec<PathBuf> = Vec::new();

    for entry in
        // Iterator over all (valid) Entries in directory
        fs::read_dir(&in_dir)?
            .flatten()
            // Filter for mkv files
            .filter(|x: &DirEntry| x.file_name().into_string().unwrap().ends_with(".mkv"))
            // Unwrap the paths
            .map(|x: DirEntry| x.path())
    {
        match rename_video(&entry) {
            Ok(_i) => renamed_videos.push(entry),
            Err(_e) => failed_videos.push(entry),
        }
    }

    if !failed_videos.is_empty() {
        println!("Failed to be rename {} videos.", failed_videos.len());
    }
    Ok(())
}

pub async fn index_videos(in_dir: PathBuf, db: &mut SqliteConnection) -> io::Result<()> {
    let files: Vec<PathBuf> = tokio::task::block_in_place(|| {
        WalkDir::new(&in_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .map_or(false, |entry| entry.eq_ignore_ascii_case("mkv"))
            })
            .map(|x| x.path().to_path_buf())
            .collect()
    });
    println!("Indexing {} videos.", files.len());
    for path in files {
        //println!("Found MKV video: {}", path.display());
        let video_path = path.display().to_string();
        if let Ok(json) = extract_json_metadata(&path) {
            sqlx::query!(
                "INSERT INTO videos (video_path, metadata) VALUES (?1, ?2)",
                video_path,
                json
            )
            .execute(db.acquire().await.unwrap())
            .await
            .unwrap();
        } else {
            continue;
        }
    }

    Ok(())
}
