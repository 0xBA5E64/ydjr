use std::{fs, path::PathBuf};

use anyhow::{Context, Ok, Result, Error};

fn extract_json_metadata(file: &PathBuf) -> Result<serde_json::Value> {
    // Parse the Matroska file
    let matroska = matroska::open(file)
        .context(format!("Unable to parse Matroska file: {:?}", file))?;
    // Find the json attachment
    let json_attachment = matroska.attachments.into_iter()
        .find(|x| x.name.ends_with(".json"))
        .ok_or(Error::msg(format!("Unable to find JSON attachment in file: {:?}", file)))?;
    // Parse it as JSON and return the result
    serde_json::from_slice(&json_attachment.data).map_err(anyhow::Error::from)
        .context(format!("Unable to parse JSON for file: {:?}", file))
}

// Renames "file" to "new_name" without changing it's directory 
fn rename_file(file: &PathBuf, new_name: &str) -> Result<()> {
    let mut new_path = file.clone();
    new_path.set_file_name(new_name);
    fs::rename(file, new_path)
        .map_err(anyhow::Error::from)
        .context(format!("Unable to rename file {:?}", file))
}

pub fn rename_video(file: PathBuf) -> Result<()> {
    // Extract JSON metadata from video-file
    let json: serde_json::Value = extract_json_metadata(&file)?;

    // Construct new filename
    let new_filename = format!(
        "{} [{}].mkv",
        json["title"].as_str().unwrap(),
        json["id"].as_str().unwrap()
    );

    // Finally, rename the file to the right filename.
    rename_file(&file, &new_filename)
}

pub fn rename_videos(in_dir: PathBuf) -> Result<()> {
    for entry in
        // Iterator over all (valid) Entries in directory
        fs::read_dir(in_dir)?.flatten()
        // Filter for mkv files
        .filter(|x| x.file_name().into_string().unwrap().ends_with(".mkv"))
        // Unwrap the paths
        .map(|x|x.path())
    {
        rename_video(entry)?
    }
    Ok(())
}

