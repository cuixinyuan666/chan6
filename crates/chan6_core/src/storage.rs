use crate::model::{ChipAccumulator, ChipBin, ChipLevel, KLine1m};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create db parent dir failed: {}", parent.display()))?;
        }
    }

    let conn = Connection::open(path).with_context(|| format!("open sqlite failed: {}", path.display()))?;
    init_db(&conn)?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS kline_1m (
            symbol TEXT NOT NULL,
            bar_id INTEGER NOT NULL,
            trading_day INTEGER NOT NULL,
            minute INTEGER NOT NULL,
            start_ts INTEGER NOT NULL,
            end_ts INTEGER NOT NULL,
            open REAL NOT NULL,
            high REAL NOT NULL,
            low REAL NOT NULL,
            close REAL NOT NULL,
            volume REAL NOT NULL,
            amount REAL NOT NULL,
            trade_count INTEGER NOT NULL,
            PRIMARY KEY(symbol, bar_id)
        );

        CREATE INDEX IF NOT EXISTS idx_kline_1m_symbol_day
        ON kline_1m(symbol, trading_day);

        CREATE TABLE IF NOT EXISTS chip_delta_1m (
            symbol TEXT NOT NULL,
            bar_id INTEGER NOT NULL,
            price_tick INTEGER NOT NULL,
            volume_delta REAL NOT NULL,
            amount_delta REAL NOT NULL,
            trade_count_delta INTEGER NOT NULL,
            PRIMARY KEY(symbol, bar_id, price_tick)
        );

        CREATE INDEX IF NOT EXISTS idx_chip_delta_symbol_bar
        ON chip_delta_1m(symbol, bar_id);

        CREATE TABLE IF NOT EXISTS chip_snapshot (
            symbol TEXT NOT NULL,
            bar_id INTEGER NOT NULL,
            bins_blob BLOB NOT NULL,
            PRIMARY KEY(symbol, bar_id)
        );
        "#,
    )?;
    Ok(())
}

pub fn set_metadata(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO metadata(key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn read_price_scale(conn: &Connection) -> Result<f64> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'price_scale'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    Ok(value
        .as_deref()
        .unwrap_or("1000")
        .parse::<f64>()
        .unwrap_or(1000.0))
}

pub fn delete_symbol_data(conn: &Connection, symbol: &str) -> Result<()> {
    conn.execute("DELETE FROM kline_1m WHERE symbol = ?1", params![symbol])?;
    conn.execute("DELETE FROM chip_delta_1m WHERE symbol = ?1", params![symbol])?;
    conn.execute("DELETE FROM chip_snapshot WHERE symbol = ?1", params![symbol])?;
    Ok(())
}

pub fn insert_kline_1m(conn: &Connection, k: &KLine1m) -> Result<()> {
    conn.execute(
        r#"
        INSERT OR REPLACE INTO kline_1m(
            symbol, bar_id, trading_day, minute, start_ts, end_ts,
            open, high, low, close, volume, amount, trade_count
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            &k.symbol,
            k.bar_id,
            k.trading_day,
            k.minute,
            k.start_ts,
            k.end_ts,
            k.open,
            k.high,
            k.low,
            k.close,
            k.volume,
            k.amount,
            k.trade_count as i64,
        ],
    )?;
    Ok(())
}

pub fn insert_chip_delta_1m(
    conn: &Connection,
    symbol: &str,
    bar_id: i64,
    delta: &BTreeMap<i64, ChipBin>,
) -> Result<()> {
    let mut stmt = conn.prepare(
        r#"
        INSERT OR REPLACE INTO chip_delta_1m(
            symbol, bar_id, price_tick, volume_delta, amount_delta, trade_count_delta
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )?;

    for (price_tick, bin) in delta {
        stmt.execute(params![
            symbol,
            bar_id,
            *price_tick,
            bin.volume,
            bin.amount,
            bin.trade_count as i64,
        ])?;
    }

    Ok(())
}

pub fn insert_chip_snapshot(
    conn: &Connection,
    symbol: &str,
    bar_id: i64,
    acc: &ChipAccumulator,
    price_scale: f64,
) -> Result<()> {
    let levels = acc.to_levels(price_scale);
    let blob = serde_json::to_vec(&levels)?;

    conn.execute(
        "INSERT OR REPLACE INTO chip_snapshot(symbol, bar_id, bins_blob) VALUES (?1, ?2, ?3)",
        params![symbol, bar_id, blob],
    )?;
    Ok(())
}

pub fn query_kline_1m(
    conn: &Connection,
    symbol: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<KLine1m>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT symbol, bar_id, trading_day, minute, start_ts, end_ts,
               open, high, low, close, volume, amount, trade_count
        FROM kline_1m
        WHERE symbol = ?1
        ORDER BY bar_id ASC
        LIMIT ?2 OFFSET ?3
        "#,
    )?;

    let rows = stmt.query_map(params![symbol, limit, offset], |row| {
        let trade_count: i64 = row.get(12)?;
        Ok(KLine1m {
            symbol: row.get(0)?,
            bar_id: row.get(1)?,
            trading_day: row.get(2)?,
            minute: row.get(3)?,
            start_ts: row.get(4)?,
            end_ts: row.get(5)?,
            open: row.get(6)?,
            high: row.get(7)?,
            low: row.get(8)?,
            close: row.get(9)?,
            volume: row.get(10)?,
            amount: row.get(11)?,
            trade_count: trade_count as u32,
        })
    })?;

    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn query_chip_state(conn: &Connection, symbol: &str, target_bar_id: i64) -> Result<Vec<ChipLevel>> {
    let price_scale = read_price_scale(conn)?;
    let (snapshot_bar_id, mut acc) = load_nearest_snapshot(conn, symbol, target_bar_id)?;
    apply_deltas(conn, symbol, snapshot_bar_id + 1, target_bar_id, &mut acc)?;
    Ok(acc.to_levels(price_scale))
}

fn load_nearest_snapshot(
    conn: &Connection,
    symbol: &str,
    target_bar_id: i64,
) -> Result<(i64, ChipAccumulator)> {
    let row: Option<(i64, Vec<u8>)> = conn
        .query_row(
            r#"
            SELECT bar_id, bins_blob
            FROM chip_snapshot
            WHERE symbol = ?1 AND bar_id <= ?2
            ORDER BY bar_id DESC
            LIMIT 1
            "#,
            params![symbol, target_bar_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    let Some((bar_id, blob)) = row else {
        return Ok((-1, ChipAccumulator::default()));
    };

    let levels: Vec<ChipLevel> = serde_json::from_slice(&blob)?;
    let mut acc = ChipAccumulator::default();
    for level in &levels {
        acc.add_level(level);
    }
    Ok((bar_id, acc))
}

fn apply_deltas(
    conn: &Connection,
    symbol: &str,
    from_bar_id: i64,
    to_bar_id: i64,
    acc: &mut ChipAccumulator,
) -> Result<()> {
    if from_bar_id > to_bar_id {
        return Ok(());
    }

    let mut stmt = conn.prepare(
        r#"
        SELECT price_tick,
               SUM(volume_delta) AS volume_delta,
               SUM(amount_delta) AS amount_delta,
               SUM(trade_count_delta) AS trade_count_delta
        FROM chip_delta_1m
        WHERE symbol = ?1 AND bar_id BETWEEN ?2 AND ?3
        GROUP BY price_tick
        ORDER BY price_tick ASC
        "#,
    )?;

    let rows = stmt.query_map(params![symbol, from_bar_id, to_bar_id], |row| {
        let trade_count: i64 = row.get(3)?;
        Ok((
            row.get::<_, i64>(0)?,
            ChipBin {
                volume: row.get(1)?,
                amount: row.get(2)?,
                trade_count: trade_count as u32,
            },
        ))
    })?;

    for row in rows {
        let (price_tick, delta) = row?;
        acc.add_bin_delta(price_tick, &delta);
    }

    Ok(())
}
