use anyhow::{bail, Result};
use chan6_core::{
    import_ticks_csv_to_sqlite, query_chip_state, query_kline_1m, query_kline_1m_at,
    storage::open_db, ImportConfig,
};
use clap::{Parser, Subcommand};
use rusqlite::Connection;
use serde_json::json;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "chan6")]
#[command(about = "Offline A-share tick processor: 1m kline + tick-accumulated chip profile")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ImportTick {
        #[arg(long)]
        csv: PathBuf,
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long, default_value_t = 1000.0)]
        price_scale: f64,
        #[arg(long, default_value_t = 60)]
        snapshot_interval: i64,
        #[arg(long, default_value_t = false)]
        replace: bool,
    },
    ImportDir {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long)]
        db: PathBuf,
        #[arg(long, default_value_t = false)]
        recursive: bool,
        #[arg(long, default_value_t = 1000.0)]
        price_scale: f64,
        #[arg(long, default_value_t = 60)]
        snapshot_interval: i64,
        #[arg(long, default_value_t = false)]
        replace: bool,
    },
    ListSymbols {
        #[arg(long)]
        db: PathBuf,
    },
    DbStats {
        #[arg(long)]
        db: PathBuf,
    },
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
    QueryKlineAt {
        #[arg(long)]
        db: PathBuf,

        #[arg(long)]
        symbol: String,

        #[arg(long)]
        day: i32,

        #[arg(long)]
        minute: i32,
    },

    QueryChip {
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        bar_id: i64,
        #[arg(long, default_value_t = 0)]
        top: usize,
    },
    QueryChipAt {
        #[arg(long)]
        db: PathBuf,

        #[arg(long)]
        symbol: String,

        #[arg(long)]
        day: i32,

        #[arg(long)]
        minute: i32,

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
        Commands::ImportDir {
            dir,
            db,
            recursive,
            price_scale,
            snapshot_interval,
            replace,
        } => {
            let files = collect_tick_files(&dir, recursive)?;
            if files.is_empty() {
                bail!("no csv/txt files found in {}", dir.display());
            }

            eprintln!("found {} files under {}", files.len(), dir.display());

            let total = files.len();
            let mut reports = Vec::new();
            let mut skipped = Vec::new();
            let mut failed = Vec::new();
            let mut replaced_symbols = HashSet::new();

            for (idx, csv) in files.into_iter().enumerate() {
                let symbol = infer_symbol_from_file_name(&csv);
                let replace_this_file = if replace {
                    match symbol.as_deref() {
                        Some(s) => replaced_symbols.insert(s.to_string()),
                        None => true,
                    }
                } else {
                    false
                };

                eprintln!("[{}/{}] importing {}", idx + 1, total, csv.display());

                match import_ticks_csv_to_sqlite(ImportConfig {
                    csv_path: csv.clone(),
                    db_path: db.clone(),
                    default_symbol: symbol,
                    price_scale,
                    snapshot_interval,
                    replace_symbol: replace_this_file,
                }) {
                    Ok(report) => {
                        if report.skipped {
                            eprintln!("[{}/{}] skipped", idx + 1, total);
                            skipped.push(json!({
                                "file": csv.display().to_string(),
                                "replace_symbol_before_import": replace_this_file,
                                "report": report,
                            }));
                        } else {
                            eprintln!("[{}/{}] ok", idx + 1, total);
                            reports.push(json!({
                                "file": csv.display().to_string(),
                                "replace_symbol_before_import": replace_this_file,
                                "report": report,
                            }));
                        }
                    }
                    Err(err) => {
                        eprintln!("[{}/{}] failed: {}", idx + 1, total, err);
                        failed.push(json!({
                            "file": csv.display().to_string(),
                            "error": err.to_string(),
                        }));
                    }
                }
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "db_path": db.display().to_string(),
                    "imported_count": reports.len(),
                    "skipped_count": skipped.len(),
                    "failed_count": failed.len(),
                    "imported": reports,
                    "skipped": skipped,
                    "failed": failed,
                }))?
            );
        }
        Commands::ListSymbols { db } => {
            let conn = open_existing_db(&db)?;
            let rows = list_symbols(&conn)?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        Commands::DbStats { db } => {
            let conn = open_existing_db(&db)?;
            let stats = db_stats(&conn, &db)?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }
        Commands::QueryKline {
            db,
            symbol,
            offset,
            limit,
        } => {
            let conn = open_existing_db(&db)?;
            let rows = query_kline_1m(&conn, &symbol, offset, limit)?;
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        Commands::QueryKlineAt {
            db,
            symbol,
            day,
            minute,
        } => {
            let conn = open_existing_db(&db)?;
            let row = query_kline_1m_at(&conn, &symbol, day, minute)?;
            println!("{}", serde_json::to_string_pretty(&row)?);
        }
        Commands::QueryChip {
            db,
            symbol,
            bar_id,
            top,
        } => {
            let conn = open_existing_db(&db)?;
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
        Commands::QueryChipAt {
            db,
            symbol,
            day,
            minute,
            top,
        } => {
            let conn = open_existing_db(&db)?;
            let kline = query_kline_1m_at(&conn, &symbol, day, minute)?;

            if let Some(kline) = kline {
                let mut levels = query_chip_state(&conn, &symbol, kline.bar_id)?;

                if top > 0 {
                    levels.sort_by(|a, b| {
                        b.volume
                            .partial_cmp(&a.volume)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    levels.truncate(top);
                    levels.sort_by_key(|x| x.price_tick);
                }

                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "symbol": &kline.symbol,
                        "bar_id": kline.bar_id,
                        "trading_day": kline.trading_day,
                        "minute": kline.minute,
                        "start_ts": kline.start_ts,
                        "end_ts": kline.end_ts,
                        "open": kline.open,
                        "high": kline.high,
                        "low": kline.low,
                        "close": kline.close,
                        "volume": kline.volume,
                        "amount": kline.amount,
                        "trade_count": kline.trade_count,
                        "chip": levels,
                    }))?
                );
            } else {
                println!("null");
            }
        }
    }

    Ok(())
}

fn open_existing_db(path: &Path) -> Result<Connection> {
    if !path.exists() {
        bail!(
            "database does not exist: {}. Run import-tick or import-dir first.",
            path.display()
        );
    }
    open_db(path)
}

fn collect_tick_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        bail!("directory does not exist: {}", dir.display());
    }
    if !dir.is_dir() {
        bail!("not a directory: {}", dir.display());
    }
    let mut out = Vec::new();
    collect_tick_files_inner(dir, recursive, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_tick_files_inner(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_tick_files_inner(&path, recursive, out)?;
            }
        } else if is_supported_tick_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_supported_tick_file(path: &Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| matches!(x.to_ascii_lowercase().as_str(), "csv" | "txt"))
        .unwrap_or(false)
}

fn infer_symbol_from_file_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let digits: String = stem.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 6 {
        Some(digits[digits.len() - 6..].to_string())
    } else if !stem.is_empty() {
        Some(stem.to_string())
    } else {
        None
    }
}

fn list_symbols(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT symbol FROM kline_1m ORDER BY symbol ASC")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn db_stats(conn: &Connection, db: &Path) -> Result<serde_json::Value> {
    let price_scale = read_metadata(conn, "price_scale")?.unwrap_or_else(|| "1000".to_string());
    let snapshot_interval =
        read_metadata(conn, "snapshot_interval")?.unwrap_or_else(|| "60".to_string());
    let symbols = list_symbol_stats(conn)?;
    Ok(json!({
        "db_path": db.display().to_string(),
        "price_scale": price_scale,
        "snapshot_interval": snapshot_interval,
        "symbols": symbols,
    }))
}

fn read_metadata(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM metadata WHERE key = ?1")?;
    let mut rows = stmt.query([key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

fn list_symbol_stats(conn: &Connection) -> Result<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            k.symbol,
            COUNT(*) AS kline_count,
            MIN(k.bar_id) AS min_bar_id,
            MAX(k.bar_id) AS max_bar_id,
            MIN(k.trading_day) AS min_trading_day,
            MAX(k.trading_day) AS max_trading_day,
            COALESCE(d.chip_delta_rows, 0) AS chip_delta_rows,
            COALESCE(s.snapshot_count, 0) AS snapshot_count,
            COALESCE(f.source_file_count, 0) AS source_file_count
        FROM kline_1m k
        LEFT JOIN (
            SELECT symbol, COUNT(*) AS chip_delta_rows
            FROM chip_delta_1m
            GROUP BY symbol
        ) d ON d.symbol = k.symbol
        LEFT JOIN (
            SELECT symbol, COUNT(*) AS snapshot_count
            FROM chip_snapshot
            GROUP BY symbol
        ) s ON s.symbol = k.symbol
        LEFT JOIN (
            SELECT symbol, COUNT(*) AS source_file_count
            FROM source_files
            GROUP BY symbol
        ) f ON f.symbol = k.symbol
        GROUP BY k.symbol
        ORDER BY k.symbol ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(json!({
            "symbol": row.get::<_, String>(0)?,
            "kline_count": row.get::<_, i64>(1)?,
            "min_bar_id": row.get::<_, Option<i64>>(2)?,
            "max_bar_id": row.get::<_, Option<i64>>(3)?,
            "min_trading_day": row.get::<_, Option<i32>>(4)?,
            "max_trading_day": row.get::<_, Option<i32>>(5)?,
            "chip_delta_rows": row.get::<_, i64>(6)?,
            "snapshot_count": row.get::<_, i64>(7)?,
            "source_file_count": row.get::<_, i64>(8)?,
        }))
    })?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}
