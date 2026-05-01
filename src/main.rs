use clap::{Parser, Subcommand};
use sqlx::migrate;
use sqlx::sqlite::*;
use std::path::PathBuf;
use ydjr::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct YdjrArgs {
    /// Where to write or open the database file from
    #[arg(long, default_value = "./db.sqlite")]
    db: PathBuf,

    /// Remove videos from db if no longer found on filesystem
    #[arg(long, short, default_value = "false")]
    remove_missing: bool,

    /// Headless mode, fitting if invoked automatically
    #[arg(long, short = 'H')]
    headless: bool,

    #[command(subcommand)]
    cmd: CmdOption,
}

#[derive(Subcommand)]
#[command()]
enum CmdOption {
    /// Index a directory into the database
    #[command(name = "index")]
    IndexDirectory { path: PathBuf },
    /// Retry failed indexings found in the database
    #[command(name = "retry-failed")]
    ReIndexFailed,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let multi_progress = indicatif::MultiProgress::new();

    let logger = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .build();

    let logger_wrapper = indicatif_log_bridge::LogWrapper::new(multi_progress.clone(), logger);

    logger_wrapper.try_init()?;

    let args = YdjrArgs::parse();

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

    match args.cmd {
        CmdOption::IndexDirectory { path } => {
            index_videos_recursively(
                path,
                &db_pool,
                args.remove_missing,
                args.headless,
                multi_progress,
            )
            .await?
        }
        CmdOption::ReIndexFailed => {
            reindex_failed_videos(&db_pool, args.remove_missing, args.headless, multi_progress)
                .await?
        }
    }
    Ok(())
}
