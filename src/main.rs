use std::{fs, path::{Path, PathBuf}};

use ytdl_json_renamer::*;

fn main() {
    let dir = PathBuf::from("./test-files");
    recursive_rename(dir)
}
