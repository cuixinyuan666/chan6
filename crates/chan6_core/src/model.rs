use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: String,
    pub ts: i64,
    /// Original CSV row order. It preserves open/close ordering when multiple ticks share the same second.
    pub seq: u64,
    pub trading_day: i32,
    pub minute: i32,
    pub price_tick: i64,
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLine1m {
    pub symbol: String,
    pub bar_id: i64,
    pub trading_day: i32,
    pub minute: i32,
    pub start_ts: i64,
    pub end_ts: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChipBin {
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipLevel {
    pub price_tick: i64,
    pub price: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ChipAccumulator {
    pub bins: BTreeMap<i64, ChipBin>,
}

impl ChipAccumulator {
    pub fn add_tick(&mut self, tick: &Tick) {
        let bin = self.bins.entry(tick.price_tick).or_default();
        bin.volume += tick.volume;
        bin.amount += tick.amount;
        bin.trade_count += 1;
    }

    pub fn add_level(&mut self, level: &ChipLevel) {
        let bin = self.bins.entry(level.price_tick).or_default();
        bin.volume += level.volume;
        bin.amount += level.amount;
        bin.trade_count += level.trade_count;
    }

    pub fn add_bin_delta(&mut self, price_tick: i64, delta: &ChipBin) {
        let bin = self.bins.entry(price_tick).or_default();
        bin.volume += delta.volume;
        bin.amount += delta.amount;
        bin.trade_count += delta.trade_count;
    }

    pub fn to_levels(&self, price_scale: f64) -> Vec<ChipLevel> {
        self.bins
            .iter()
            .map(|(price_tick, bin)| ChipLevel {
                price_tick: *price_tick,
                price: *price_tick as f64 / price_scale,
                volume: bin.volume,
                amount: bin.amount,
                trade_count: bin.trade_count,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ImportConfig {
    pub csv_path: PathBuf,
    pub db_path: PathBuf,
    pub default_symbol: Option<String>,
    pub price_scale: f64,
    pub snapshot_interval: i64,
    pub replace_symbol: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolImportReport {
    pub symbol: String,
    pub tick_count: usize,
    pub kline_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub db_path: String,
    pub price_scale: f64,
    pub snapshot_interval: i64,
    pub symbols: Vec<SymbolImportReport>,
}
