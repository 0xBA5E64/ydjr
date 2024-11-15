use std::path::PathBuf;

use anyhow::Result;
use ytdl_json_renamer::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of file/files to rename
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let dir = PathBuf::from(args.path);

    rename_videos(dir)
}
