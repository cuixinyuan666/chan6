use anyhow::Result;
use chan6_core::{
    import_ticks_csv_to_sqlite, query_chip_state, query_kline_1m, storage::open_db, ImportConfig,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "chan6")]
#[command(about = "Offline A-share tick processor: 1m kline + tick-accumulated chip profile")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Import offline tick CSV into SQLite cache.
    ImportTick {
        /// Offline tick CSV path.
        #[arg(long)]
        csv: PathBuf,

        /// SQLite cache path.
        #[arg(long)]
        db: PathBuf,

        /// Symbol used when CSV has no symbol/code column.
        #[arg(long)]
        symbol: Option<String>,

        /// Price scale. 1000 means 10.235 -> price_tick 10235.
        #[arg(long, default_value_t = 1000.0)]
        price_scale: f64,

        /// Save one full chip snapshot every N bars.
        #[arg(long, default_value_t = 60)]
        snapshot_interval: i64,

        /// Delete existing rows of imported symbols before writing new data.
        #[arg(long, default_value_t = false)]
        replace: bool,
    },

    /// Query generated 1m klines.
    QueryKline {
        #[arg(long)]
        db: PathBuf,

        #[arg(long)]
        symbol: String,

        #[arg(long, default_value_t = 0)]
        offset: i64,

        #[arg(long, default_value_t = 100)]
        limit: i64,
    },

    /// Query tick-accumulated chip state at a specific 1m bar.
    QueryChip {
        #[arg(long)]
        db: PathBuf,

        #[arg(long)]
        symbol: String,

        #[arg(long)]
        bar_id: i64,

        /// Return only top N levels by volume. 0 means all price levels.
        #[arg(long, default_value_t = 0)]
        top: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ImportTick {
            csv,
            db,
            symbol,
            price_scale,
            snapshot_interval,
            replace,
        } => {
            let report = import_ticks_csv_to_sqlite(ImportConfig {
                csv_path: csv,
                db_path: db,
                default_symbol: symbol,
                price_scale,
                snapshot_interval,
                replace_symbol: replace,
            })?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::QueryKline {
            db,
            symbol,
            offset,
            limit,
        } => {
            let conn = open_db(&db)?;
            let rows = query_kline_1m(&conn, &symbol, offset, limit)?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        Commands::QueryChip {
            db,
            symbol,
            bar_id,
            top,
        } => {
            let conn = open_db(&db)?;
            let mut levels = query_chip_state(&conn, &symbol, bar_id)?;

            if top > 0 {
                levels.sort_by(|a, b| {
                    b.volume
                        .partial_cmp(&a.volume)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                levels.truncate(top);
                levels.sort_by_key(|x| x.price_tick);
            }

            println!("{}", serde_json::to_string_pretty(&levels)?);
        }
    }

    Ok(())
}
