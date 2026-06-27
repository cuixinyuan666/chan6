use anyhow::{bail, Result};
use chan6_core::{
    chan::{analyze_chan_basic_with_config, ChanConfig},
    query_kline_1m,
};
use clap::Parser;
use rusqlite::Connection;
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "query_chan_basic")]
#[command(about = "Query kline data and Rust Chan stage-1 include/fx/bi overlay data")]
struct Args {
    #[arg(long)]
    db: PathBuf,

    #[arg(long)]
    symbol: String,

    #[arg(long, default_value_t = 0)]
    offset: i64,

    #[arg(long, default_value_t = 300)]
    limit: i64,

    #[arg(long, default_value = "1m")]
    level: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if !args.db.exists() {
        bail!("database does not exist: {}", args.db.display());
    }

    let conn = Connection::open(&args.db)?;
    let kline = query_kline_1m(&conn, &args.symbol, args.offset, args.limit)?;
    let chan_basic = analyze_chan_basic_with_config(&kline, &args.level, &ChanConfig::default());

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "meta": {
                "schema_version": 1,
                "query": "query-chan-basic",
                "symbol": args.symbol,
                "level": args.level,
                "kline_scope": "offset_limit_window",
                "offset": args.offset,
                "limit": args.limit,
                "kline_count": kline.len(),
                "merged_count": chan_basic.merged_bars.len(),
                "fx_count": chan_basic.fx.len(),
                "bi_count": chan_basic.bi.len(),
                "render_hint": {
                    "merged_box": "draw rectangle by start_bar_id/end_bar_id/high/low",
                    "fx_line": "draw polyline by fx.bar_id/fx.price in fx order",
                    "calc_fields": "calc_high/calc_low are internal algorithm fields; high/low are visual envelope"
                }
            },
            "kline": kline,
            "chan_basic": chan_basic,
            "merged_boxes": chan_basic.merged_bars.iter().map(|bar| json!({
                "index": bar.index,
                "start_bar_id": bar.start_bar_id,
                "end_bar_id": bar.end_bar_id,
                "high": bar.high,
                "low": bar.low,
                "is_merged": bar.start_bar_id != bar.end_bar_id,
                "high_bar_id": bar.high_bar_id,
                "low_bar_id": bar.low_bar_id,
                "calc_high": bar.calc_high,
                "calc_low": bar.calc_low,
                "calc_high_bar_id": bar.calc_high_bar_id,
                "calc_low_bar_id": bar.calc_low_bar_id
            })).collect::<Vec<_>>(),
            "fx_lines": chan_basic.fx.iter().map(|fx| json!({
                "index": fx.index,
                "kind": fx.kind,
                "merged_index": fx.merged_index,
                "bar_id": fx.bar_id,
                "price": fx.price,
                "confirmed": fx.confirmed
            })).collect::<Vec<_>>(),
            "bi_lines": chan_basic.bi.iter().map(|bi| json!({
                "index": bi.index,
                "direction": bi.direction,
                "start_bar_id": bi.start_bar_id,
                "start_price": bi.start_price,
                "end_bar_id": bi.end_bar_id,
                "end_price": bi.end_price,
                "confirmed": bi.confirmed
            })).collect::<Vec<_>>()
        }))?
    );

    Ok(())
}
