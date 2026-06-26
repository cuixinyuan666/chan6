use crate::model::Tick;
use crate::session::{a_share_1m_bar_info, parse_tick_datetime};
use anyhow::{anyhow, Context, Result};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use csv::StringRecord;
use encoding_rs::GBK;
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TickTextReadOptions {
    pub default_symbol: Option<String>,
    pub price_scale: f64,
}

#[derive(Debug, Clone)]
struct Columns {
    symbol: Option<usize>,
    time: usize,
    price: usize,
    volume: usize,
    amount: Option<usize>,
}

pub fn read_ticks_from_text(path: &Path, opt: &TickTextReadOptions) -> Result<Vec<Tick>> {
    let raw = read_text(path)?;
    let default_date = date_from_path(path).or_else(|| date_from_head(&raw));
    let prepared = prepare_table(path, &raw)?;

    let mut rdr = csv::ReaderBuilder::new()
        .flexible(true)
        .delimiter(prepared.delimiter)
        .from_reader(prepared.text.as_bytes());

    let headers = rdr.headers()?.clone();
    let cols = detect_columns(&headers, opt.default_symbol.is_some()).with_context(|| {
        format!(
            "detect columns failed for {}. headers={:?}, skipped_lines={}",
            path.display(),
            headers,
            prepared.skipped_lines
        )
    })?;

    let mut ticks = Vec::new();
    for (idx, row) in rdr.records().enumerate() {
        let row =
            row.with_context(|| format!("read row {} failed", idx + prepared.skipped_lines + 2))?;
        if let Some(tick) = parse_row(idx as u64, &row, &cols, opt, default_date)
            .with_context(|| format!("parse row {} failed", idx + prepared.skipped_lines + 2))?
        {
            ticks.push(tick);
        }
    }

    ticks.sort_by(|a, b| {
        a.symbol
            .cmp(&b.symbol)
            .then(a.ts.cmp(&b.ts))
            .then(a.seq.cmp(&b.seq))
    });
    Ok(ticks)
}

struct PreparedTable {
    text: String,
    delimiter: u8,
    skipped_lines: usize,
}

fn read_text(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("open tick text failed: {}", path.display()))?;
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

fn prepare_table(path: &Path, raw: &str) -> Result<PreparedTable> {
    let lines: Vec<&str> = raw.lines().collect();
    let header_idx = lines
        .iter()
        .position(|line| looks_like_header(line))
        .ok_or_else(|| {
            anyhow!(
                "cannot find header line in {}. first lines: {}",
                path.display(),
                preview(raw)
            )
        })?;

    let delimiter0 = delimiter_of(lines[header_idx]);
    let use_tab = delimiter0 == b' ';
    let text = lines[header_idx..]
        .iter()
        .map(|line| {
            if use_tab {
                line.trim()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join("\t")
            } else {
                line.trim().to_string()
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(PreparedTable {
        text,
        delimiter: if use_tab { b'\t' } else { delimiter0 },
        skipped_lines: header_idx,
    })
}

fn looks_like_header(line: &str) -> bool {
    let n = norm(line);
    contains_any(
        &n,
        &[
            "datetime",
            "time",
            "\u{6210}\u{4ea4}\u{65f6}\u{95f4}",
            "\u{65f6}\u{95f4}",
            "\u{65e5}\u{671f}\u{65f6}\u{95f4}",
        ],
    ) && contains_any(
        &n,
        &[
            "price",
            "\u{6210}\u{4ea4}\u{4ef7}",
            "\u{6210}\u{4ea4}\u{4ef7}\u{683c}",
            "\u{6700}\u{65b0}\u{4ef7}",
            "\u{4ef7}\u{683c}",
        ],
    ) && contains_any(
        &n,
        &[
            "volume",
            "vol",
            "qty",
            "\u{6210}\u{4ea4}\u{91cf}",
            "\u{6210}\u{4ea4}",
            "\u{6570}\u{91cf}",
            "\u{6210}\u{4ea4}\u{6570}\u{91cf}",
            "\u{624b}\u{6570}",
        ],
    )
}

fn contains_any(n: &str, words: &[&str]) -> bool {
    words.iter().any(|w| n.contains(&norm(w)))
}

fn delimiter_of(line: &str) -> u8 {
    let candidates = [(b'\t', '\t'), (b',', ','), (b';', ';'), (b'|', '|')];
    let mut best = (b' ', 0usize);
    for (b, c) in candidates {
        let cnt = line.matches(c).count();
        if cnt > best.1 {
            best = (b, cnt);
        }
    }
    best.0
}

fn detect_columns(headers: &StringRecord, has_default_symbol: bool) -> Result<Columns> {
    let symbol = find(
        headers,
        &[
            "symbol",
            "code",
            "\u{80a1}\u{7968}\u{4ee3}\u{7801}",
            "\u{8bc1}\u{5238}\u{4ee3}\u{7801}",
            "\u{4ee3}\u{7801}",
            "\u{8bc1}\u{5238}",
        ],
    );
    let time = find(
        headers,
        &[
            "datetime",
            "time",
            "\u{6210}\u{4ea4}\u{65f6}\u{95f4}",
            "\u{65f6}\u{95f4}",
            "\u{65e5}\u{671f}\u{65f6}\u{95f4}",
        ],
    )
    .ok_or_else(|| anyhow!("missing time column"))?;
    let price = find(
        headers,
        &[
            "price",
            "\u{6210}\u{4ea4}\u{4ef7}",
            "\u{6210}\u{4ea4}\u{4ef7}\u{683c}",
            "\u{6700}\u{65b0}\u{4ef7}",
            "\u{4ef7}\u{683c}",
        ],
    )
    .ok_or_else(|| anyhow!("missing price column"))?;
    let volume = find(
        headers,
        &[
            "volume",
            "vol",
            "qty",
            "\u{6210}\u{4ea4}\u{91cf}",
            "\u{6210}\u{4ea4}",
            "\u{6570}\u{91cf}",
            "\u{6210}\u{4ea4}\u{6570}\u{91cf}",
            "\u{624b}\u{6570}",
        ],
    )
    .ok_or_else(|| anyhow!("missing volume column"))?;
    let amount = find(
        headers,
        &[
            "amount",
            "\u{6210}\u{4ea4}\u{989d}",
            "\u{6210}\u{4ea4}\u{91d1}\u{989d}",
            "\u{91d1}\u{989d}",
        ],
    );

    if symbol.is_none() && !has_default_symbol {
        return Err(anyhow!("missing symbol column and no default symbol"));
    }

    Ok(Columns {
        symbol,
        time,
        price,
        volume,
        amount,
    })
}

fn find(headers: &StringRecord, names: &[&str]) -> Option<usize> {
    headers.iter().position(|h| {
        let h = norm(h);
        names.iter().any(|name| h == norm(name))
    })
}

fn norm(s: &str) -> String {
    s.trim()
        .trim_start_matches('\u{feff}')
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !matches!(c, ' ' | '_' | '-' | '\t'))
        .collect()
}

fn parse_row(
    seq: u64,
    row: &StringRecord,
    cols: &Columns,
    opt: &TickTextReadOptions,
    default_date: Option<NaiveDate>,
) -> Result<Option<Tick>> {
    let symbol = match cols.symbol {
        Some(i) => cell(row, i)?.trim().to_string(),
        None => opt.default_symbol.clone().unwrap(),
    };
    if symbol.is_empty() {
        return Ok(None);
    }

    let dt = parse_time_cell(cell(row, cols.time)?, default_date)?;
    let Some(bar) = a_share_1m_bar_info(dt) else {
        return Ok(None);
    };

    let price = num(cell(row, cols.price)?)?;
    let volume = num(cell(row, cols.volume)?)?;
    if price <= 0.0 || volume <= 0.0 {
        return Ok(None);
    }

    let amount = match cols.amount {
        Some(i) => {
            let s = cell(row, i)?.trim();
            if s.is_empty() {
                price * volume
            } else {
                num(s)?
            }
        }
        None => price * volume,
    };

    Ok(Some(Tick {
        symbol,
        ts: dt.and_utc().timestamp(),
        seq,
        trading_day: bar.trading_day,
        minute: bar.minute,
        price_tick: (price * opt.price_scale).round() as i64,
        price,
        volume,
        amount,
    }))
}

fn parse_time_cell(raw: &str, default_date: Option<NaiveDate>) -> Result<NaiveDateTime> {
    if let Ok(dt) = parse_tick_datetime(raw) {
        return Ok(dt);
    }
    let date = default_date.ok_or_else(|| anyhow!("time-only row has no date: {raw}"))?;
    for fmt in ["%H:%M:%S", "%H:%M", "%H%M%S", "%H%M", "%H:%M:%S%.3f"] {
        if let Ok(t) = NaiveTime::parse_from_str(raw.trim(), fmt) {
            return Ok(date.and_time(t));
        }
    }
    Err(anyhow!("unsupported time format: {raw}"))
}

fn cell(row: &StringRecord, i: usize) -> Result<&str> {
    row.get(i).ok_or_else(|| anyhow!("column {i} out of range"))
}

fn num(raw: &str) -> Result<f64> {
    let s = raw.trim().replace(',', "");
    if s.is_empty() || s == "--" || s == "-" {
        return Err(anyhow!("empty number"));
    }
    Ok(s.parse::<f64>()?)
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

fn preview(raw: &str) -> String {
    raw.lines()
        .take(8)
        .map(|x| x.trim())
        .collect::<Vec<_>>()
        .join(" | ")
}
