use chan6_core::chan::analyze_chan_basic;
use chan6_core::chan::model::ChanDirection;
use chan6_core::model::KLine1m;
use serde::Deserialize;

#[test]
fn chanpy_stage1_bi_endpoint_pullback_updates_to_later_stronger_top() {
    let symbol = "stage1_bi_endpoint_pullback_candidate";
    let csv = include_str!(
        "../../../fixtures/chanpy_stage1/input/stage1_bi_endpoint_pullback_candidate.csv"
    );
    let gold_text = include_str!("../../../fixtures/chanpy_stage1/gold/stage1_bi_endpoint_pullback_candidate_chanpy_gold.json");

    let gold: Stage1Gold = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(symbol, csv);
    let snapshot = analyze_chan_basic(&klines);

    assert_eq!(snapshot.meta.merged_count, gold.meta.merged_bars_count);
    assert_eq!(snapshot.meta.fx_count, gold.meta.fx_count);
    assert_eq!(snapshot.meta.bi_count, gold.meta.bi_count);
    assert_eq!(snapshot.meta.segment_count, gold.meta.segment_count);

    assert_eq!(snapshot.bi.len(), gold.bi.len());

    let actual_tail = snapshot.bi.last().expect("rust tail bi");
    let expected_tail = gold.bi.last().expect("gold tail bi");

    assert_eq!(actual_tail.index, expected_tail.index);
    assert_eq!(actual_tail.start_bar_id, expected_tail.start_raw_index);
    assert_eq!(actual_tail.end_bar_id, expected_tail.end_raw_index);
    assert_close(actual_tail.start_price, expected_tail.start_price);
    assert_close(actual_tail.end_price, expected_tail.end_price);
    assert_eq!(actual_tail.confirmed, expected_tail.is_sure);

    match expected_tail.direction.as_str() {
        "up" => assert_eq!(actual_tail.direction, ChanDirection::Up),
        "down" => assert_eq!(actual_tail.direction, ChanDirection::Down),
        other => panic!("unexpected gold bi direction: {other}"),
    }

    assert_eq!(expected_tail.start_raw_index, 135);
    assert_eq!(expected_tail.end_raw_index, 156);
    assert_close(expected_tail.start_price, 7.5);
    assert_close(expected_tail.end_price, 26.0);
    assert_eq!(expected_tail.direction, "up");
}

fn parse_stage1_csv(symbol: &str, csv_text: &str) -> Vec<KLine1m> {
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
                symbol: symbol.to_string(),
                bar_id: index as i64,
                trading_day: 20270407 + index as i32,
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
    bi: Vec<GoldBi>,
}

#[derive(Debug, Deserialize)]
struct Stage1GoldMeta {
    merged_bars_count: usize,
    fx_count: usize,
    bi_count: usize,
    segment_count: usize,
}

#[derive(Debug, Deserialize)]
struct GoldBi {
    index: usize,
    direction: String,
    start_raw_index: i64,
    end_raw_index: i64,
    start_price: f64,
    end_price: f64,
    is_sure: bool,
}
