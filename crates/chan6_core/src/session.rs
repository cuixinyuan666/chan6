use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

#[derive(Debug, Clone, Copy)]
pub struct BarMinuteInfo {
    pub trading_day: i32,
    pub minute: i32,
    pub start_ts: i64,
    pub end_ts: i64,
}

pub fn parse_tick_datetime(raw: &str) -> Result<NaiveDateTime> {
    let s = raw.trim();

    const FORMATS: [&str; 7] = [
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y%m%d %H:%M:%S",
        "%Y%m%d%H%M%S",
        "%Y-%m-%d %H:%M:%S%.3f",
        "%Y/%m/%d %H:%M:%S%.3f",
        "%Y%m%d %H:%M:%S%.3f",
    ];

    for fmt in FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(dt);
        }
    }

    Err(anyhow!("unsupported datetime format: {s}"))
}

pub fn trading_day(dt: NaiveDateTime) -> i32 {
    let d = dt.date();
    d.year() * 10000 + d.month() as i32 * 100 + d.day() as i32
}

pub fn a_share_1m_bar_info(dt: NaiveDateTime) -> Option<BarMinuteInfo> {
    let seconds = dt.hour() as i32 * 3600 + dt.minute() as i32 * 60 + dt.second() as i32;

    let morning_start = 9 * 3600 + 30 * 60;
    let morning_end = 11 * 3600 + 30 * 60;
    let afternoon_start = 13 * 3600;
    let afternoon_end = 15 * 3600;

    let bucket_dt = if seconds >= morning_start && seconds < morning_end {
        dt.with_second(0)?.with_nanosecond(0)?
    } else if seconds == morning_end {
        // 保留 11:30:00 收盘边界成交，归入 11:29 这一根 start-minute K。
        dt.date().and_hms_opt(11, 29, 0)?
    } else if seconds >= afternoon_start && seconds < afternoon_end {
        dt.with_second(0)?.with_nanosecond(0)?
    } else if seconds == afternoon_end {
        // 保留 15:00:00 收盘边界成交，归入 14:59 这一根 start-minute K。
        dt.date().and_hms_opt(14, 59, 0)?
    } else {
        return None;
    };

    Some(BarMinuteInfo {
        trading_day: trading_day(dt),
        minute: bucket_dt.hour() as i32 * 100 + bucket_dt.minute() as i32,
        start_ts: bucket_dt.and_utc().timestamp(),
        end_ts: bucket_dt.and_utc().timestamp() + 59,
    })
}

pub fn date_from_trading_day(day: i32) -> Option<NaiveDate> {
    let y = day / 10000;
    let m = ((day / 100) % 100) as u32;
    let d = (day % 100) as u32;
    NaiveDate::from_ymd_opt(y, m, d)
}
