use serde::{Deserialize, Serialize};

use crate::model::KLine1m;

use super::bi::build_bis;
use super::config::ChanConfig;
use super::fx::detect_fxs;
use super::include::merge_included_bars;
use super::model::{ChanBar, ChanBi, ChanFx, ChanMergedBar};

pub const CHAN_BASIC_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanBasicMeta {
    pub query: String,
    pub schema_version: u32,
    pub symbol: String,
    pub level: String,
    pub kline_count: usize,
    pub merged_count: usize,
    pub fx_count: usize,
    pub bi_count: usize,
    pub include_mode: String,
    pub fx_mode: String,
    pub bi_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChanBasicSnapshot {
    pub meta: ChanBasicMeta,
    pub bars: Vec<ChanBar>,
    pub merged_bars: Vec<ChanMergedBar>,
    pub fx: Vec<ChanFx>,
    pub bi: Vec<ChanBi>,
}

pub fn analyze_chan_basic(klines: &[KLine1m]) -> ChanBasicSnapshot {
    analyze_chan_basic_with_config(klines, "1m", &ChanConfig::default())
}

pub fn analyze_chan_basic_with_config(
    klines: &[KLine1m],
    level: &str,
    config: &ChanConfig,
) -> ChanBasicSnapshot {
    let bars: Vec<ChanBar> = klines.iter().map(ChanBar::from).collect();
    let merged_bars = merge_included_bars(&bars);
    let fx = detect_fxs(&merged_bars);
    let bi = build_bis(&fx);

    let symbol = bars
        .first()
        .map(|bar| bar.symbol.clone())
        .or_else(|| klines.first().map(|bar| bar.symbol.clone()))
        .unwrap_or_default();

    let meta = ChanBasicMeta {
        query: "query-chan-basic".to_string(),
        schema_version: CHAN_BASIC_SCHEMA_VERSION,
        symbol,
        level: level.to_string(),
        kline_count: bars.len(),
        merged_count: merged_bars.len(),
        fx_count: fx.len(),
        bi_count: bi.len(),
        include_mode: format!("{:?}", config.include_mode).to_lowercase(),
        fx_mode: format!("{:?}", config.fx_mode).to_lowercase(),
        bi_mode: format!("{:?}", config.bi_mode).to_lowercase(),
    };

    ChanBasicSnapshot {
        meta,
        bars,
        merged_bars,
        fx,
        bi,
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_chan_basic, analyze_chan_basic_with_config, CHAN_BASIC_SCHEMA_VERSION};
    use crate::chan::config::ChanConfig;
    use crate::chan::model::{ChanDirection, ChanFxKind};
    use crate::model::KLine1m;

    #[test]
    fn empty_input_returns_empty_snapshot_with_stable_meta() {
        let snapshot = analyze_chan_basic(&[]);

        assert_eq!(snapshot.meta.query, "query-chan-basic");
        assert_eq!(snapshot.meta.schema_version, CHAN_BASIC_SCHEMA_VERSION);
        assert_eq!(snapshot.meta.kline_count, 0);
        assert_eq!(snapshot.meta.merged_count, 0);
        assert_eq!(snapshot.meta.fx_count, 0);
        assert_eq!(snapshot.meta.bi_count, 0);
        assert!(snapshot.bars.is_empty());
        assert!(snapshot.merged_bars.is_empty());
        assert!(snapshot.fx.is_empty());
        assert!(snapshot.bi.is_empty());
    }

    #[test]
    fn pipeline_builds_bars_merged_fxs_and_bis() {
        let klines = vec![
            kline(1, 10.0, 8.0),
            kline(2, 12.0, 10.0),
            kline(3, 11.0, 7.0),
            kline(4, 13.0, 9.0),
            kline(5, 12.0, 8.0),
        ];

        let snapshot = analyze_chan_basic(&klines);

        assert_eq!(snapshot.meta.symbol, "002003");
        assert_eq!(snapshot.meta.level, "1m");
        assert_eq!(snapshot.meta.kline_count, 5);
        assert_eq!(snapshot.meta.merged_count, 5);
        assert_eq!(snapshot.meta.fx_count, 3);
        assert_eq!(snapshot.meta.bi_count, 2);
        assert_eq!(snapshot.fx[0].kind, ChanFxKind::Top);
        assert_eq!(snapshot.fx[1].kind, ChanFxKind::Bottom);
        assert_eq!(snapshot.fx[2].kind, ChanFxKind::Top);
        assert_eq!(snapshot.bi[0].direction, ChanDirection::Down);
        assert_eq!(snapshot.bi[1].direction, ChanDirection::Up);
        assert_eq!(snapshot.bi[0].next_index, Some(1));
        assert_eq!(snapshot.bi[1].prev_index, Some(0));
    }

    #[test]
    fn pipeline_runs_include_before_fx_detection() {
        let klines = vec![
            kline(1, 10.0, 8.0),
            kline(2, 11.0, 9.0),
            kline(3, 10.5, 9.5),
            kline(4, 12.0, 8.0),
            kline(5, 11.0, 7.0),
        ];

        let snapshot = analyze_chan_basic(&klines);

        assert_eq!(snapshot.meta.kline_count, 5);
        assert!(snapshot.meta.merged_count < snapshot.meta.kline_count);
        assert!(snapshot
            .merged_bars
            .iter()
            .any(|bar| bar.start_bar_id == 2 && bar.end_bar_id == 3));
    }

    #[test]
    fn config_and_custom_level_are_reflected_in_meta() {
        let klines = vec![kline(1, 10.0, 8.0), kline(2, 11.0, 9.0)];
        let config = ChanConfig::default();

        let snapshot = analyze_chan_basic_with_config(&klines, "daily", &config);

        assert_eq!(snapshot.meta.level, "daily");
        assert_eq!(snapshot.meta.include_mode, "standard");
        assert_eq!(snapshot.meta.fx_mode, "strict");
        assert_eq!(snapshot.meta.bi_mode, "normal");
    }

    fn kline(bar_id: i64, high: f64, low: f64) -> KLine1m {
        KLine1m {
            symbol: "002003".to_string(),
            bar_id,
            trading_day: 20260511,
            minute: 930 + bar_id as i32,
            start_ts: bar_id * 60,
            end_ts: bar_id * 60 + 59,
            open: low,
            high,
            low,
            close: high,
            volume: 100.0,
            amount: 1000.0,
            trade_count: 10,
        }
    }
}
