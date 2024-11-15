use std::{fs, path::PathBuf};

fn extract_json_metadata(file: &PathBuf) -> serde_json::Value {
    let matroska = matroska::open(file).unwrap();
    let json_attachment = matroska.attachments.into_iter().find(|x| x.name.eq("info.json")).unwrap();
    serde_json::from_slice(&json_attachment.data).unwrap()
}

fn rename_file(file: &PathBuf, new_name: &str) {
    let mut new_path = file.clone();
    new_path.set_file_name(new_name);
    //println!("Renaming {} to {}", entry.path().file_name().unwrap().to_str().unwrap(), new_path.file_name().unwrap().to_str().unwrap());
    fs::rename(file, new_path).unwrap()
}

pub fn rename_video(file: &PathBuf) {
    let json: serde_json::Value = extract_json_metadata(file);

    let new_filename = format!(
        "{} [{}].mkv",
        json["title"].as_str().unwrap(),
        json["id"].as_str().unwrap()
    );

    rename_file(&file, &new_filename);
}

pub fn rename_videos(in_dir: PathBuf) {
    let paths = fs::read_dir(in_dir).unwrap();
    for entry in paths
        .filter_map(|x| Some(x.unwrap()) )
        .filter(|p| p.file_name().into_string().unwrap().ends_with(".mkv") )
    {
        rename_video(&entry.path())
    }

}

