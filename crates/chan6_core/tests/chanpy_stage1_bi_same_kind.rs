use chan6_core::chan::{analyze_chan_basic, ChanDirection, ChanFxKind};
use chan6_core::model::KLine1m;
use serde::Deserialize;

#[test]
fn chanpy_stage1_bi_same_kind_replacement_gold_matches_rust_pipeline() {
    assert_chanpy_gold_matches_rust_pipeline(
        include_str!("../../../fixtures/chanpy_stage1/input/stage1_bi_same_kind_replacement_candidate.csv"),
        include_str!("../../../fixtures/chanpy_stage1/gold/stage1_bi_same_kind_replacement_candidate_chanpy_gold.json"),
    );
}

fn assert_chanpy_gold_matches_rust_pipeline(csv_text: &str, gold_text: &str) {
    let klines = parse_stage1_csv(csv_text);
    let gold: Stage1Gold = serde_json::from_str(gold_text).expect("stage1 chan.py gold json must parse");

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
    assert!((actual - expected).abs() < 1e-9, "actual={actual}, expected={expected}");
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
