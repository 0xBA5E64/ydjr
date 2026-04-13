use clap::Parser;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{migrate, ConnectOptions};
use std::io;
use std::path::PathBuf;
use ydjr::index_videos;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of file/files to rename
    path: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
    let dir = PathBuf::from(args.path);

    let mut db = SqliteConnectOptions::new()
        .filename("db.sqlite")
        .create_if_missing(true)
        .connect()
        .await
        .unwrap();

    migrate!("./migrations").run(&mut db).await.unwrap();

    println!("db: {:?}", db);

    index_videos(dir, &mut db).await
}
