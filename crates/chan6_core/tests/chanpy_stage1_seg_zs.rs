use chan6_core::chan::analyze_chan_basic;
use chan6_core::model::KLine1m;
use serde::Deserialize;

#[test]
fn chanpy_stage1_zs_overlap_probe_seg_zs_gold_matches_rust_pipeline() {
    assert_candidate(
        "stage1_zs_overlap_probe_candidate",
        include_str!("../../../fixtures/chanpy_stage1/input/stage1_zs_overlap_probe_candidate.csv"),
        include_str!(
            "../../../fixtures/chanpy_stage1/gold/stage1_zs_overlap_probe_candidate_chanpy_gold.json"
        ),
    );
}

#[test]
fn chanpy_stage1_segseg_probe_seg_zs_gold_matches_rust_pipeline() {
    assert_candidate(
        "stage1_segseg_probe_candidate",
        include_str!("../../../fixtures/chanpy_stage1/input/stage1_segseg_probe_candidate.csv"),
        include_str!(
            "../../../fixtures/chanpy_stage1/gold/stage1_segseg_probe_candidate_chanpy_gold.json"
        ),
    );
}

fn assert_candidate(symbol: &str, csv: &str, gold_text: &str) {
    let gold: Stage1Gold = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(symbol, csv);
    let snapshot = analyze_chan_basic(&klines);

    assert_eq!(snapshot.meta.merged_count, gold.meta.merged_bars_count);
    assert_eq!(snapshot.meta.fx_count, gold.meta.fx_count);
    assert_eq!(snapshot.meta.bi_count, gold.meta.bi_count);
    assert_eq!(snapshot.meta.segment_count, gold.meta.segment_count);

    assert_eq!(gold.meta.seg_zs_count, 1);
    assert_eq!(gold.seg_zs.len(), gold.meta.seg_zs_count);
    assert_eq!(snapshot.seg_zs.len(), gold.seg_zs.len());

    for (actual, expected) in snapshot.seg_zs.iter().zip(&gold.seg_zs) {
        assert_eq!(actual.index, expected.index);
        assert_eq!(actual.start_bar_id, expected.start_raw_index.unwrap());
        assert_eq!(actual.end_bar_id, expected.end_raw_index.unwrap());
        assert_close(actual.zg, expected.zg);
        assert_close(actual.zd, expected.zd);
        assert_close(actual.gg, expected.gg);
        assert_close(actual.dd, expected.dd);
    }

    let first = &snapshot.seg_zs[0];
    assert_eq!(first.start_segment_index, 1);
    assert_eq!(first.end_segment_index, 5);
    assert_eq!(first.start_bar_id, 13);
    assert_eq!(first.end_bar_id, 116);
    assert_close(first.zg, 15.0);
    assert_close(first.zd, 2.5);
    assert_close(first.gg, 18.5);
    assert_close(first.dd, 0.5);
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
    seg_zs: Vec<GoldSegZs>,
}

#[derive(Debug, Deserialize)]
struct Stage1GoldMeta {
    merged_bars_count: usize,
    fx_count: usize,
    bi_count: usize,
    segment_count: usize,
    seg_zs_count: usize,
}

#[derive(Debug, Deserialize)]
struct GoldSegZs {
    index: usize,
    start_raw_index: Option<i64>,
    end_raw_index: Option<i64>,
    zg: f64,
    zd: f64,
    gg: f64,
    dd: f64,
}
