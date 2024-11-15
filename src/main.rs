use std::path::PathBuf;

use ytdl_json_renamer::*;

fn main() {
    let dir = PathBuf::from("./test-files");
    rename_videos(dir)
}
