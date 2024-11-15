use std::{fs, path::PathBuf};

pub fn extract_json_metadata(file: PathBuf) -> serde_json::Value {
    let matroska = matroska::open(file).unwrap();
    let json_attachment = matroska.attachments.into_iter().find(|x| x.name.eq("info.json")).unwrap();
    serde_json::from_slice(&json_attachment.data).unwrap()
}

fn rename_file(file: PathBuf, new_name: &str) {
    let mut new_path = file.clone();
    new_path.set_file_name(new_name);
    //println!("Renaming {} to {}", entry.path().file_name().unwrap().to_str().unwrap(), new_path.file_name().unwrap().to_str().unwrap());
    fs::rename(file, new_path).unwrap()
}

pub fn recursive_rename(in_dir: PathBuf) {
    let paths = fs::read_dir(in_dir).unwrap();
    for entry in paths.map(|p| p.unwrap()) {

        if entry.file_name().into_string().unwrap().ends_with(".mkv") {
            println!("Found MKV-file: {}", entry.file_name().into_string().unwrap());
            
            let json = extract_json_metadata(entry.path());

            let new_filename = format!(
                "{} [{}].mkv",
                json["title"].as_str().unwrap(),
                json["id"].as_str().unwrap()
            );

            rename_file(entry.path(), &new_filename);
            
        } else {
            println!("BAD FILE: {:?}", entry)
        }
    }

}

