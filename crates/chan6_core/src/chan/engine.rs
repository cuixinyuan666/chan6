use serde::{Deserialize, Serialize};

use crate::model::KLine1m;

use super::bi::build_bis_with_merged_bars;
use super::config::ChanConfig;
use super::fx::detect_fxs;
use super::include::merge_included_bars;
use super::model::{ChanBar, ChanBi, ChanFx, ChanMergedBar, ChanSegment};
use super::segment::build_segments;
use super::zs::{build_seg_zs, build_zs, ChanSegZs, ChanZs};

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
    pub segment_count: usize,
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
    pub segments: Vec<ChanSegment>,
    pub zs: Vec<ChanZs>,
    pub seg_zs: Vec<ChanSegZs>,
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
    let bi = build_bis_with_merged_bars(&fx, &merged_bars);
    let segments = build_segments(&bi);
    let zs = build_zs(&bi, &segments);
    let seg_zs = build_seg_zs(&segments);

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
        segment_count: segments.len(),
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
        segments,
        zs,
        seg_zs,
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_chan_basic, analyze_chan_basic_with_config, CHAN_BASIC_SCHEMA_VERSION};
    use crate::chan::config::ChanConfig;
    use crate::chan::model::{ChanDirection, ChanFxKind};
    use crate::model::KLine1m;
    use serde::Deserialize;

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
    fn pipeline_builds_bars_merged_and_fxs() {
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
        assert_eq!(snapshot.meta.bi_count, 0);
        assert_eq!(snapshot.fx[0].kind, ChanFxKind::Top);
        assert_eq!(snapshot.fx[1].kind, ChanFxKind::Bottom);
        assert_eq!(snapshot.fx[2].kind, ChanFxKind::Top);
        assert!(snapshot.bi.is_empty());
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
            .any(|bar| bar.start_bar_id <= 2 && bar.end_bar_id >= 3));
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

    #[test]
    fn chanpy_stage1_small_gold_matches_rust_pipeline() {
        assert_chanpy_gold_matches_rust_pipeline(
            include_str!("../../../../fixtures/chanpy_stage1/input/stage1_small.csv"),
            include_str!("../../../../fixtures/chanpy_stage1/gold/stage1_small_chanpy_gold.json"),
        );
    }

    #[test]
    fn chanpy_stage1_bi_candidate_gold_matches_rust_pipeline() {
        assert_chanpy_gold_matches_rust_pipeline(
            include_str!("../../../../fixtures/chanpy_stage1/input/stage1_bi_candidate.csv"),
            include_str!(
                "../../../../fixtures/chanpy_stage1/gold/stage1_bi_candidate_chanpy_gold.json"
            ),
        );
    }

    #[test]
    fn chanpy_stage1_end_is_peak_fail_gold_matches_rust_pipeline() {
        assert_chanpy_gold_matches_rust_pipeline(
            include_str!(
                "../../../../fixtures/chanpy_stage1/input/stage1_end_is_peak_fail_candidate.csv"
            ),
            include_str!(
                "../../../../fixtures/chanpy_stage1/gold/stage1_end_is_peak_fail_candidate_chanpy_gold.json"
            ),
        );
    }

    fn assert_chanpy_gold_matches_rust_pipeline(csv_text: &str, gold_text: &str) {
        let klines = parse_stage1_csv(csv_text);
        let gold: Stage1Gold =
            serde_json::from_str(gold_text).expect("stage1 chan.py gold json must parse");

        let snapshot = analyze_chan_basic(&klines);

        assert!(gold.ok);
        assert_eq!(snapshot.bars.len(), gold.meta.bars_count);
        assert_eq!(snapshot.merged_bars.len(), gold.merged_bars.len());
        assert_eq!(snapshot.fx.len(), gold.fx.len());
        assert_eq!(snapshot.bi.len(), gold.bi.len());
        assert_eq!(snapshot.meta.merged_count, gold.meta.merged_bars_count);
        assert_eq!(snapshot.meta.fx_count, gold.meta.fx_count);
        assert_eq!(snapshot.meta.bi_count, gold.meta.bi_count);

        for (actual, expected) in snapshot.merged_bars.iter().zip(&gold.merged_bars) {
            assert_eq!(actual.index, expected.index);
            assert_eq!(actual.start_bar_id, expected.start_raw_index);
            assert_eq!(actual.end_bar_id, expected.end_raw_index);
            assert_eq!(actual.high_bar_id, expected.high_raw_index);
            assert_eq!(actual.low_bar_id, expected.low_raw_index);
            assert_close(actual.open, expected.open);
            assert_close(actual.high, expected.high);
            assert_close(actual.low, expected.low);
            assert_close(actual.close, expected.close);
        }

        for (actual, expected) in snapshot.fx.iter().zip(&gold.fx) {
            assert_eq!(actual.merged_index, expected.index);
            assert_eq!(actual.bar_id, expected.raw_index);
            assert_close(actual.price, expected.price);
            match expected.fx_type.as_str() {
                "top" => assert_eq!(actual.kind, ChanFxKind::Top),
                "bottom" => assert_eq!(actual.kind, ChanFxKind::Bottom),
                other => panic!("unexpected gold fx type: {other}"),
            }
        }

        for (actual, expected) in snapshot.bi.iter().zip(&gold.bi) {
            assert_eq!(actual.index, expected.index);
            assert_eq!(actual.start_bar_id, expected.start_raw_index);
            assert_eq!(actual.end_bar_id, expected.end_raw_index);
            assert_close(actual.start_price, expected.start_price);
            assert_close(actual.end_price, expected.end_price);
            assert_eq!(actual.confirmed, expected.is_sure);
            match expected.direction.as_str() {
                "up" => assert_eq!(actual.direction, ChanDirection::Up),
                "down" => assert_eq!(actual.direction, ChanDirection::Down),
                other => panic!("unexpected gold bi direction: {other}"),
            }
        }
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

    fn parse_stage1_csv(text: &str) -> Vec<KLine1m> {
        text.lines()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .enumerate()
            .map(|(index, line)| {
                let parts: Vec<&str> = line.split(',').collect();
                assert_eq!(parts.len(), 6, "unexpected csv row: {line}");
                let open = parse_f64(parts[1]);
                let raw_high = parse_f64(parts[2]);
                let raw_low = parse_f64(parts[3]);
                let close = parse_f64(parts[4]);
                let high = open.max(raw_high).max(raw_low).max(close);
                let low = open.min(raw_high).min(raw_low).min(close);
                let volume = parse_f64(parts[5]);
                KLine1m {
                    symbol: "stage1_fixture".to_string(),
                    bar_id: index as i64,
                    trading_day: parts[0].replace('-', "").parse::<i32>().unwrap(),
                    minute: 0,
                    start_ts: index as i64,
                    end_ts: index as i64,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    amount: 0.0,
                    trade_count: 0,
                }
            })
            .collect()
    }

    fn parse_f64(text: &str) -> f64 {
        text.parse::<f64>().unwrap()
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "actual={actual}, expected={expected}"
        );
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Stage1Gold {
        ok: bool,
        meta: Stage1GoldMeta,
        merged_bars: Vec<Stage1GoldMergedBar>,
        fx: Vec<Stage1GoldFx>,
        bi: Vec<Stage1GoldBi>,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct Stage1GoldMeta {
        engine: String,
        symbol: String,
        freq: String,
        adjust: String,
        mode: String,
        gold_source: String,
        generated_by: String,
        bars_count: usize,
        merged_bars_count: usize,
        fx_count: usize,
        bi_count: usize,
    }

    #[derive(Debug, Deserialize)]
    struct Stage1GoldMergedBar {
        index: usize,
        start_raw_index: i64,
        end_raw_index: i64,
        high_raw_index: i64,
        low_raw_index: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
    }

    #[derive(Debug, Deserialize)]
    struct Stage1GoldFx {
        index: usize,
        raw_index: i64,
        #[serde(rename = "type")]
        fx_type: String,
        price: f64,
    }

    #[derive(Debug, Deserialize)]
    struct Stage1GoldBi {
        index: usize,
        start_raw_index: i64,
        end_raw_index: i64,
        start_price: f64,
        end_price: f64,
        direction: String,
        is_sure: bool,
    }
}
