use anyhow::Context;
use clap::{Parser, Subcommand};
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
    /// Add mediainfo to older entries missing it
    #[command(name = "add-mediainfo")]
    AddMediainfo,
    /// Print json metadata of file to console without any indexing
    #[command(name = "print-json")]
    PrintJson { path: PathBuf },
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

    match args.cmd {
        CmdOption::IndexDirectory { path } => {
            let db_pool = initiate_database(args.db).await;
            index_videos_recursively(
                path.clone(),
                &db_pool,
                args.remove_missing,
                args.headless,
                multi_progress,
            )
            .await
            .with_context(|| {
                format!(
                    "Failed at indexing directory: \"{}\"",
                    path.to_string_lossy()
                )
            })
        }
        CmdOption::ReIndexFailed => {
            let db_pool = initiate_database(args.db).await;
            reindex_failed_videos(&db_pool, args.remove_missing, args.headless, multi_progress)
                .await
                .context("Failed to reindex failed videos")
        }
        CmdOption::AddMediainfo => {
            let db_pool = initiate_database(args.db).await;
            amend_empty_mediainfo(&db_pool, args.headless, multi_progress)
                .await
                .context("Failed to add mediainfo to older entries")
        }
        CmdOption::PrintJson { path } => {
            println!("{}", extract_json_metadata(&path)?);
            Ok(())
        }
    }
}
