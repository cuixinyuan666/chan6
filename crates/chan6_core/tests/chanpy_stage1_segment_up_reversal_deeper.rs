use chan6_core::chan::analyze_chan_basic;
use chan6_core::chan::model::{ChanDirection, CHAN_SEGMENT_N_LINE};
use chan6_core::model::KLine1m;
use serde::Deserialize;

#[test]
fn chanpy_stage1_segment_up_reversal_deeper_gold_matches_main_pipeline() {
    let csv = include_str!(
        "../../../fixtures/chanpy_stage1/input/stage1_segment_up_reversal_deeper_candidate.csv"
    );
    let gold_text = include_str!(
        "../../../fixtures/chanpy_stage1/gold/stage1_segment_up_reversal_deeper_candidate_chanpy_gold.json"
    );
    let gold: Stage1Gold = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(csv);
    let snapshot = analyze_chan_basic(&klines);

    assert_eq!(snapshot.meta.merged_count, gold.meta.merged_bars_count);
    assert_eq!(snapshot.meta.fx_count, gold.meta.fx_count);
    assert_eq!(snapshot.meta.bi_count, gold.meta.bi_count);
    assert_eq!(snapshot.meta.segment_count, gold.meta.segment_count);

    assert_eq!(snapshot.fx.len(), gold.meta.fx_count);
    assert_eq!(snapshot.bi.len(), gold.meta.bi_count);
    assert_eq!(snapshot.segments.len(), gold.meta.segment_count);
    assert_eq!(snapshot.segments.len(), gold.seg.len());

    for (actual, expected) in snapshot.segments.iter().zip(&gold.seg) {
        assert_eq!(actual.index, expected.index);
        assert_eq!(actual.n, CHAN_SEGMENT_N_LINE);
        assert_eq!(actual.input_n, None);
        assert_eq!(actual.start_parent_index, Some(expected.start_bi_index));
        assert_eq!(actual.end_parent_index, Some(expected.end_bi_index));
        assert_eq!(actual.start_bar_id, expected.start_raw_index);
        assert_eq!(actual.end_bar_id, expected.end_raw_index);
        assert_close(actual.start_price, expected.start_price);
        assert_close(actual.end_price, expected.end_price);
        assert_eq!(actual.confirmed, expected.is_sure);

        match expected.direction.as_str() {
            "up" => assert_eq!(actual.direction, ChanDirection::Up),
            "down" => assert_eq!(actual.direction, ChanDirection::Down),
            other => panic!("unexpected gold segment direction: {other}"),
        }
    }

    assert_eq!(snapshot.segments.len(), 2);

    assert_eq!(snapshot.segments[0].start_parent_index, Some(0));
    assert_eq!(snapshot.segments[0].end_parent_index, Some(2));
    assert_eq!(snapshot.segments[0].start_bar_id, 1);
    assert_eq!(snapshot.segments[0].end_bar_id, 13);
    assert_eq!(snapshot.segments[0].end_price, 15.0);
    assert_eq!(snapshot.segments[0].direction, ChanDirection::Up);
    assert!(!snapshot.segments[0].confirmed);

    assert_eq!(snapshot.segments[1].start_parent_index, Some(3));
    assert_eq!(snapshot.segments[1].end_parent_index, Some(9));
    assert_eq!(snapshot.segments[1].start_bar_id, 13);
    assert_eq!(snapshot.segments[1].end_bar_id, 41);
    assert_eq!(snapshot.segments[1].start_price, 15.0);
    assert_eq!(snapshot.segments[1].end_price, 2.5);
    assert_eq!(snapshot.segments[1].direction, ChanDirection::Down);
    assert!(!snapshot.segments[1].confirmed);
}

fn parse_stage1_csv(csv_text: &str) -> Vec<KLine1m> {
    csv_text
        .lines()
        .skip(1)
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let cols: Vec<_> = line.split(',').collect();
            Some(KLine1m {
                symbol: "stage1_segment_up_reversal_deeper_candidate".to_string(),
                bar_id: index as i64,
                trading_day: 20270210 + index as i32,
                minute: 0,
                start_ts: index as i64,
                end_ts: index as i64,
                open: cols[1].parse().unwrap(),
                high: cols[2].parse().unwrap(),
                low: cols[3].parse().unwrap(),
                close: cols[4].parse().unwrap(),
                volume: cols[5].parse().unwrap(),
                amount: 0.0,
                trade_count: 0,
            })
        })
        .collect()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "actual={actual}, expected={expected}"
    );
}

#[derive(Debug, Deserialize)]
struct Stage1Gold {
    meta: Stage1GoldMeta,
    seg: Vec<Stage1GoldSegment>,
}

#[derive(Debug, Deserialize)]
struct Stage1GoldMeta {
    merged_bars_count: usize,
    fx_count: usize,
    bi_count: usize,
    segment_count: usize,
}

#[derive(Debug, Deserialize)]
struct Stage1GoldSegment {
    index: usize,
    start_bi_index: usize,
    end_bi_index: usize,
    start_raw_index: i64,
    end_raw_index: i64,
    start_price: f64,
    end_price: f64,
    direction: String,
    is_sure: bool,
}
