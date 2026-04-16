use clap::Parser;
use sqlx::migrate;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqlitePoolOptions};
use std::path::PathBuf;
use ydjr::index_videos;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of file/files to rename
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Where to write or open the database file from
    #[arg(long, default_value = "./db.sqlite")]
    db: PathBuf,

    /// Headless mode, fitting if invoked automatically
    #[arg(long, short = 'H')]
    headless: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let db_pool = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(args.db)
                .create_if_missing(true)
                .auto_vacuum(SqliteAutoVacuum::Incremental),
        )
        .await
        .expect("Unable to establish database connection");

    migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to apply database migrations");

    index_videos(args.path, &db_pool, args.headless).await?;
    Ok(())
}
