use crate::csv_reader::{read_ticks_from_csv, TickCsvReadOptions};
use crate::model::{ChipAccumulator, ChipBin, ImportConfig, ImportReport, KLine1m, SymbolImportReport, Tick};
use crate::session::date_from_trading_day;
use crate::storage::{delete_symbol_data, insert_chip_delta_1m, insert_chip_snapshot, insert_kline_1m, open_db, set_metadata};
use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeMap;

pub fn import_ticks_csv_to_sqlite(config: ImportConfig) -> Result<ImportReport> {
    if config.price_scale <= 0.0 {
        return Err(anyhow!("price_scale must be greater than 0"));
    }
    if config.snapshot_interval <= 0 {
        return Err(anyhow!("snapshot_interval must be greater than 0"));
    }

    let ticks = read_ticks_from_csv(
        &config.csv_path,
        &TickCsvReadOptions {
            default_symbol: config.default_symbol.clone(),
            price_scale: config.price_scale,
        },
    )?;

    let conn = open_db(&config.db_path)?;
    conn.execute_batch("BEGIN IMMEDIATE")?;

    let result = import_sorted_ticks(&conn, &ticks, &config);

    match result {
        Ok(report) => {
            conn.execute_batch("COMMIT")?;
            Ok(report)
        }
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

fn import_sorted_ticks(conn: &Connection, ticks: &[Tick], config: &ImportConfig) -> Result<ImportReport> {
    set_metadata(conn, "price_scale", &config.price_scale.to_string())?;
    set_metadata(conn, "snapshot_interval", &config.snapshot_interval.to_string())?;

    let mut reports = Vec::new();
    let mut start = 0usize;

    while start < ticks.len() {
        let symbol = ticks[start].symbol.clone();
        let mut end = start + 1;
        while end < ticks.len() && ticks[end].symbol == symbol {
            end += 1;
        }

        if config.replace_symbol {
            delete_symbol_data(conn, &symbol)?;
        }

        let start_bar_id = next_bar_id_for_symbol(conn, &symbol)?;
        let report = process_symbol_ticks(
            conn,
            &symbol,
            &ticks[start..end],
            config.price_scale,
            config.snapshot_interval,
            start_bar_id,
        )?;
        reports.push(report);
        start = end;
    }

    Ok(ImportReport {
        db_path: config.db_path.display().to_string(),
        price_scale: config.price_scale,
        snapshot_interval: config.snapshot_interval,
        symbols: reports,
    })
}

fn next_bar_id_for_symbol(conn: &Connection, symbol: &str) -> Result<i64> {
    let max_id: Option<i64> = conn
        .query_row(
            "SELECT MAX(bar_id) FROM kline_1m WHERE symbol = ?1",
            params![symbol],
            |row| row.get(0),
        )
        .optional()?;
    Ok(max_id.map(|x| x + 1).unwrap_or(0))
}

fn process_symbol_ticks(
    conn: &Connection,
    symbol: &str,
    ticks: &[Tick],
    price_scale: f64,
    snapshot_interval: i64,
    start_bar_id: i64,
) -> Result<SymbolImportReport> {
    let mut processor = TickProcessor::new(conn, symbol, price_scale, snapshot_interval, start_bar_id);

    for tick in ticks {
        processor.on_tick(tick)?;
    }

    processor.finish()?;

    Ok(SymbolImportReport {
        symbol: symbol.to_string(),
        tick_count: ticks.len(),
        kline_count: processor.kline_count,
    })
}

struct TickProcessor<'a> {
    conn: &'a Connection,
    symbol: String,
    price_scale: f64,
    snapshot_interval: i64,
    current_key: Option<(i32, i32)>,
    current_kline: Option<KLine1m>,
    current_delta: BTreeMap<i64, ChipBin>,
    chip_acc: ChipAccumulator,
    next_bar_id: i64,
    kline_count: usize,
}

impl<'a> TickProcessor<'a> {
    fn new(conn: &'a Connection, symbol: &str, price_scale: f64, snapshot_interval: i64, start_bar_id: i64) -> Self {
        Self {
            conn,
            symbol: symbol.to_string(),
            price_scale,
            snapshot_interval,
            current_key: None,
            current_kline: None,
            current_delta: BTreeMap::new(),
            chip_acc: ChipAccumulator::default(),
            next_bar_id: start_bar_id,
            kline_count: 0,
        }
    }

    fn on_tick(&mut self, tick: &Tick) -> Result<()> {
        let key = (tick.trading_day, tick.minute);

        if self.current_key != Some(key) {
            self.flush_current_bar()?;
            self.current_key = Some(key);
            self.current_delta.clear();
            self.current_kline = Some(new_kline_from_tick(tick, self.next_bar_id)?);
            self.next_bar_id += 1;
        } else if let Some(kline) = self.current_kline.as_mut() {
            update_kline(kline, tick);
        }

        self.chip_acc.add_tick(tick);

        let delta = self.current_delta.entry(tick.price_tick).or_default();
        delta.volume += tick.volume;
        delta.amount += tick.amount;
        delta.trade_count += 1;

        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.flush_current_bar()
    }

    fn flush_current_bar(&mut self) -> Result<()> {
        let Some(kline) = self.current_kline.take() else {
            return Ok(());
        };

        insert_kline_1m(self.conn, &kline)?;
        insert_chip_delta_1m(self.conn, &self.symbol, kline.bar_id, &self.current_delta)?;

        if kline.bar_id % self.snapshot_interval == 0 {
            insert_chip_snapshot(self.conn, &self.symbol, kline.bar_id, &self.chip_acc, self.price_scale)?;
        }

        self.kline_count += 1;
        Ok(())
    }
}

fn new_kline_from_tick(tick: &Tick, bar_id: i64) -> Result<KLine1m> {
    let date = date_from_trading_day(tick.trading_day).ok_or_else(|| anyhow!("invalid trading_day: {}", tick.trading_day))?;
    let hour = tick.minute / 100;
    let minute = tick.minute % 100;
    let start_dt = date
        .and_hms_opt(hour as u32, minute as u32, 0)
        .ok_or_else(|| anyhow!("invalid minute: {}", tick.minute))?;
    let start_ts = start_dt.and_utc().timestamp();

    Ok(KLine1m {
        symbol: tick.symbol.clone(),
        bar_id,
        trading_day: tick.trading_day,
        minute: tick.minute,
        start_ts,
        end_ts: start_ts + 59,
        open: tick.price,
        high: tick.price,
        low: tick.price,
        close: tick.price,
        volume: tick.volume,
        amount: tick.amount,
        trade_count: 1,
    })
}

fn update_kline(kline: &mut KLine1m, tick: &Tick) {
    if tick.price > kline.high {
        kline.high = tick.price;
    }
    if tick.price < kline.low {
        kline.low = tick.price;
    }
    kline.close = tick.price;
    kline.volume += tick.volume;
    kline.amount += tick.amount;
    kline.trade_count += 1;
}
