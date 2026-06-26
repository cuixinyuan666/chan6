use crate::model::Tick;
use crate::session::{a_share_1m_bar_info, parse_tick_datetime};
use anyhow::{anyhow, Context, Result};
use csv::StringRecord;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TickCsvReadOptions {
    pub default_symbol: Option<String>,
    pub price_scale: f64,
}

impl Default for TickCsvReadOptions {
    fn default() -> Self {
        Self {
            default_symbol: None,
            price_scale: 1000.0,
        }
    }
}

#[derive(Debug, Clone)]
struct CsvColumns {
    symbol: Option<usize>,
    datetime: usize,
    price: usize,
    volume: usize,
    amount: Option<usize>,
}

pub fn read_ticks_from_csv(path: &Path, options: &TickCsvReadOptions) -> Result<Vec<Tick>> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path)
        .with_context(|| format!("open tick csv failed: {}", path.display()))?;

    let headers = rdr.headers()?.clone();
    let cols = detect_columns(&headers, options.default_symbol.is_some())?;

    let mut ticks = Vec::new();
    for (row_idx, record) in rdr.records().enumerate() {
        let record = record.with_context(|| format!("read csv row {} failed", row_idx + 2))?;

        if let Some(tick) = parse_record(&record, &cols, options)
            .with_context(|| format!("parse csv row {} failed", row_idx + 2))?
        {
            ticks.push(tick);
        }
    }

    ticks.sort_by(|a, b| {
        a.symbol
            .cmp(&b.symbol)
            .then(a.ts.cmp(&b.ts))
            .then(a.price_tick.cmp(&b.price_tick))
    });

    Ok(ticks)
}

fn detect_columns(headers: &StringRecord, has_default_symbol: bool) -> Result<CsvColumns> {
    let symbol = find_header(headers, &["symbol", "code", "股票代码", "证券代码", "代码"]);
    let datetime = find_header(headers, &["datetime", "time", "成交时间", "时间", "日期时间"])
        .ok_or_else(|| anyhow!("missing datetime column; supported headers: datetime,time,成交时间,时间,日期时间"))?;
    let price = find_header(headers, &["price", "成交价", "成交价格", "最新价"])
        .ok_or_else(|| anyhow!("missing price column; supported headers: price,成交价,成交价格,最新价"))?;
    let volume = find_header(headers, &["volume", "vol", "qty", "成交量", "数量"])
        .ok_or_else(|| anyhow!("missing volume column; supported headers: volume,vol,qty,成交量,数量"))?;
    let amount = find_header(headers, &["amount", "成交额", "成交金额"]);

    if symbol.is_none() && !has_default_symbol {
        return Err(anyhow!(
            "missing symbol column; provide one of symbol,code,股票代码,证券代码,代码 or pass --symbol"
        ));
    }

    Ok(CsvColumns {
        symbol,
        datetime,
        price,
        volume,
        amount,
    })
}

fn find_header(headers: &StringRecord, names: &[&str]) -> Option<usize> {
    headers.iter().position(|h| {
        let normalized = normalize_header(h);
        names.iter().any(|name| normalized == normalize_header(name))
    })
}

fn normalize_header(s: &str) -> String {
    s.trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .replace([' ', '_', '-'], "")
}

fn parse_record(
    record: &StringRecord,
    cols: &CsvColumns,
    options: &TickCsvReadOptions,
) -> Result<Option<Tick>> {
    let symbol = match cols.symbol {
        Some(idx) => get(record, idx)?.trim().to_string(),
        None => options.default_symbol.clone().unwrap(),
    };

    if symbol.is_empty() {
        return Ok(None);
    }

    let datetime_raw = get(record, cols.datetime)?;
    let dt = parse_tick_datetime(datetime_raw)?;
    let Some(bar) = a_share_1m_bar_info(dt) else {
        // 非 A 股连续竞价时段，直接跳过。
        return Ok(None);
    };

    let price = parse_number(get(record, cols.price)?)?;
    let volume = parse_number(get(record, cols.volume)?)?;

    if price <= 0.0 || volume <= 0.0 {
        return Ok(None);
    }

    let amount = match cols.amount {
        Some(idx) => {
            let raw = get(record, idx)?.trim();
            if raw.is_empty() {
                price * volume
            } else {
                parse_number(raw)?
            }
        }
        None => price * volume,
    };

    let price_tick = (price * options.price_scale).round() as i64;

    Ok(Some(Tick {
        symbol,
        ts: dt.and_utc().timestamp(),
        trading_day: bar.trading_day,
        minute: bar.minute,
        price_tick,
        price,
        volume,
        amount,
    }))
}

fn get(record: &StringRecord, idx: usize) -> Result<&str> {
    record
        .get(idx)
        .ok_or_else(|| anyhow!("csv column index {} out of range", idx))
}

fn parse_number(raw: &str) -> Result<f64> {
    let s = raw.trim().replace(',', "");
    if s.is_empty() {
        return Err(anyhow!("empty number"));
    }
    s.parse::<f64>()
        .with_context(|| format!("invalid number: {raw}"))
}
