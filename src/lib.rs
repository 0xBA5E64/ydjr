use std::{fs, path::PathBuf};

fn extract_json_metadata(file: &PathBuf) -> serde_json::Value {
    // Parse the Matroska file
    let matroska = matroska::open(file).unwrap();
    // Find the json attachment
    let json_attachment = matroska.attachments.into_iter().find(|x| x.name.eq("info.json")).unwrap();
    // Parse it as JSON and return the result
    serde_json::from_slice(&json_attachment.data).unwrap()
}

// Renames "file" to "new_name" without changing it's directory 
fn rename_file(file: &PathBuf, new_name: &str) {
    let mut new_path = file.clone();
    new_path.set_file_name(new_name);
    fs::rename(file, new_path).unwrap()
}

pub fn rename_video(file: PathBuf) {
    // Extract JSON metadata from video-file
    let json: serde_json::Value = extract_json_metadata(&file);

    // Construct new filename
    let new_filename = format!(
        "{} [{}].mkv",
        json["title"].as_str().unwrap(),
        json["id"].as_str().unwrap()
    );

    // Finally, rename the file to the right filename.
    rename_file(&file, &new_filename);
}

pub fn rename_videos(in_dir: PathBuf) {
    for entry in
        // Iterator over all Entries in directory
        fs::read_dir(in_dir).unwrap().flatten()
        // Filter for mkv files
        .filter(|x| x.file_name().into_string().unwrap().ends_with(".mkv"))
        // Unwrap the paths
        .map(|x|x.path())
    {
        rename_video(entry)
    }
}

