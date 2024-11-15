use std::fs;

fn main() {
    let dir = "./test-files";
    let paths = fs::read_dir(dir).unwrap();
    for path in paths.map(|p| p.unwrap()) {
        if path.file_name().into_string().unwrap().ends_with(".mkv") {
            println!("Found MKV-file: {}", path.file_name().into_string().unwrap());
            let matroska = matroska::open(path.path()).unwrap();
            let json_attachment = matroska.attachments.into_iter().find(|x| x.name.eq("info.json")).unwrap();
            let json: serde_json::Value = serde_json::from_slice(&json_attachment.data).unwrap();
            
            let new_filename = format!(
                "{} [{}].mkv",
                json["title"].as_str().unwrap(),
                json["id"].as_str().unwrap()
            );

            let mut new_path = path.path().clone();
            new_path.set_file_name(new_filename);
            println!("Renaming {} to {}", path.path().file_name().unwrap().to_str().unwrap(), new_path.file_name().unwrap().to_str().unwrap());
            fs::rename(path.path(), new_path).unwrap();

        } else {
            println!("BAD FILE: {:?}", path)
        }
    }
}
