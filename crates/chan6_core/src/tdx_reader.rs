use crate::model::Tick;
use crate::session::a_share_1m_bar_info;
use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveTime};
use encoding_rs::GBK;
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TdxReadOptions {
    pub default_symbol: Option<String>,
    pub price_scale: f64,
}

pub fn read_ticks_from_tdx_text(path: &Path, opt: &TdxReadOptions) -> Result<Vec<Tick>> {
    let raw = read_text(path)?;
    let date = date_from_path(path)
        .or_else(|| date_from_head(&raw))
        .ok_or_else(|| anyhow!("cannot infer trading date from file name or title: {}", path.display()))?;
    let symbol = opt
        .default_symbol
        .clone()
        .or_else(|| symbol_from_path(path))
        .ok_or_else(|| anyhow!("cannot infer symbol from file name: {}", path.display()))?;

    let mut ticks = Vec::new();
    for (seq, line) in raw.lines().enumerate() {
        let Some(tick) = parse_tdx_line(seq as u64, line, date, &symbol, opt.price_scale)? else {
            continue;
        };
        ticks.push(tick);
    }

    ticks.sort_by(|a, b| a.ts.cmp(&b.ts).then(a.seq.cmp(&b.seq)));
    Ok(ticks)
}

fn parse_tdx_line(
    seq: u64,
    line: &str,
    date: NaiveDate,
    symbol: &str,
    price_scale: f64,
) -> Result<Option<Tick>> {
    let s = line.trim();
    if s.is_empty() || s.starts_with('#') {
        return Ok(None);
    }

    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 3 {
        return Ok(None);
    }

    let Some(time) = parse_time(parts[0]) else {
        return Ok(None);
    };

    let Ok(price) = parse_num(parts[1]) else {
        return Ok(None);
    };
    let Ok(volume) = parse_num(parts[2]) else {
        return Ok(None);
    };

    if price <= 0.0 || volume <= 0.0 {
        return Ok(None);
    }

    let dt = date.and_time(time);
    let Some(bar) = a_share_1m_bar_info(dt) else {
        return Ok(None);
    };

    let amount = price * volume;
    Ok(Some(Tick {
        symbol: symbol.to_string(),
        ts: dt.and_utc().timestamp(),
        seq,
        trading_day: bar.trading_day,
        minute: bar.minute,
        price_tick: (price * price_scale).round() as i64,
        price,
        volume,
        amount,
    }))
}

fn parse_time(raw: &str) -> Option<NaiveTime> {
    for fmt in ["%H:%M:%S", "%H:%M", "%H%M%S", "%H%M", "%H:%M:%S%.3f"] {
        if let Ok(t) = NaiveTime::parse_from_str(raw.trim(), fmt) {
            return Some(t);
        }
    }
    None
}

fn parse_num(raw: &str) -> Result<f64> {
    let s = raw.trim().replace(',', "");
    Ok(s.parse::<f64>()?)
}

fn read_text(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("open tdx text failed: {}", path.display()))?;
    match String::from_utf8(bytes) {
        Ok(s) => Ok(s),
        Err(err) => {
            let bytes = err.into_bytes();
            let (s, _, _) = GBK.decode(&bytes);
            Ok(match s {
                Cow::Borrowed(x) => x.to_string(),
                Cow::Owned(x) => x,
            })
        }
    }
}

fn date_from_path(path: &Path) -> Option<NaiveDate> {
    let stem = path.file_stem()?.to_string_lossy();
    extract_date(&stem)
}

fn date_from_head(raw: &str) -> Option<NaiveDate> {
    raw.lines().take(5).find_map(extract_date)
}

fn extract_date(s: &str) -> Option<NaiveDate> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 8 {
        return None;
    }
    for i in 0..=(digits.len() - 8) {
        if let Ok(d) = NaiveDate::parse_from_str(&digits[i..i + 8], "%Y%m%d") {
            return Some(d);
        }
    }
    None
}

fn symbol_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let digits: String = stem.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 6 {
        Some(digits[digits.len() - 6..].to_string())
    } else {
        None
    }
}
