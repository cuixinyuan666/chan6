use anyhow::{bail, Result};
use chan6_core::{build_chan_basic_overlay, query_kline_1m, storage::open_db};
use clap::Parser;
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "query_chan_basic")]
#[command(about = "Query K-line window with inclusion-merged boxes and FX lines")]
struct Args {
    #[arg(long)]
    db: PathBuf,

    #[arg(long)]
    symbol: String,

    #[arg(long, default_value_t = 0)]
    offset: i64,

    #[arg(long, default_value_t = 300)]
    limit: i64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    ensure_db_exists(&args.db)?;

    let conn = open_db(&args.db)?;
    let kline = query_kline_1m(&conn, &args.symbol, args.offset, args.limit)?;
    let chan_basic = build_chan_basic_overlay(&kline);

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "meta": {
                "schema_version": 1,
                "query": "query-chan-basic",
                "symbol": &args.symbol,
                "offset": args.offset,
                "limit": args.limit,
                "kline_count": kline.len(),
                "merged_box_count": chan_basic.merged_boxes.len(),
                "fx_point_count": chan_basic.fx_points.len(),
                "fx_line_count": chan_basic.fx_lines.len(),
            },
            "symbol": args.symbol,
            "offset": args.offset,
            "limit": args.limit,
            "kline": kline,
            "chan_basic": chan_basic,
        }))?
    );

    Ok(())
}

fn ensure_db_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!(
            "database does not exist: {}. Run import-tick or import-dir first.",
            path.display()
        );
    }
    Ok(())
}
