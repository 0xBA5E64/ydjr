use clap::Parser;
use sqlx::migrate;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqlitePoolOptions};
use std::io;
use std::path::PathBuf;
use ydjr::index_videos;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of file/files to rename
    path: PathBuf,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
    let dir: PathBuf = args.path;

    let db_pool = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(
            SqliteConnectOptions::new()
                .filename("db.sqlite")
                .create_if_missing(true)
                .auto_vacuum(SqliteAutoVacuum::Incremental),
        )
        .await
        .expect("Unable to establish database connection");

    migrate!("./migrations").run(&db_pool).await.unwrap();

    index_videos(dir, &db_pool).await
}
