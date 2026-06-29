use chan6_core::chan::analyze_chan_basic;
use chan6_core::model::KLine1m;
use serde::Deserialize;

#[test]
fn chanpy_stage1_segseg_probe_zs_gold_matches_rust_pipeline() {
    let symbol = "stage1_segseg_probe_candidate";
    let csv =
        include_str!("../../../fixtures/chanpy_stage1/input/stage1_segseg_probe_candidate.csv");
    let gold_text = include_str!(
        "../../../fixtures/chanpy_stage1/gold/stage1_segseg_probe_candidate_chanpy_gold.json"
    );

    let gold: Stage1Gold = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(symbol, csv);
    let snapshot = analyze_chan_basic(&klines);

    assert_eq!(snapshot.meta.merged_count, gold.meta.merged_bars_count);
    assert_eq!(snapshot.meta.fx_count, gold.meta.fx_count);
    assert_eq!(snapshot.meta.bi_count, gold.meta.bi_count);
    assert_eq!(snapshot.meta.segment_count, gold.meta.segment_count);

    assert_eq!(gold.meta.segseg_count, 0);
    assert_eq!(gold.meta.zs_count, 3);
    assert_eq!(gold.meta.seg_zs_count, 1);
    assert_eq!(gold.meta.bsp_count, 8);

    assert_eq!(gold.segseg.len(), gold.meta.segseg_count);
    assert_eq!(gold.zs.len(), gold.meta.zs_count);
    assert_eq!(gold.seg_zs.len(), gold.meta.seg_zs_count);
    assert_eq!(gold.bsp.len(), gold.meta.bsp_count);

    assert_eq!(snapshot.zs.len(), gold.zs.len());

    for (actual, expected) in snapshot.zs.iter().zip(&gold.zs) {
        assert_eq!(actual.index, expected.index);
        assert_eq!(actual.start_bi_index, expected.start_bi_index.unwrap());
        assert_eq!(actual.end_bi_index, expected.end_bi_index.unwrap());
        assert_eq!(actual.start_bar_id, expected.start_raw_index.unwrap());
        assert_eq!(actual.end_bar_id, expected.end_raw_index.unwrap());
        assert_close(actual.zg, expected.zg);
        assert_close(actual.zd, expected.zd);
        assert_close(actual.gg, expected.gg);
        assert_close(actual.dd, expected.dd);
    }

    let zs2 = &gold.zs[2];
    assert_eq!(zs2.index, 2);
    assert_eq!(zs2.start_bi_index, Some(27));
    assert_eq!(zs2.end_bi_index, Some(29));
    assert_eq!(zs2.start_raw_index, Some(121));
    assert_eq!(zs2.end_raw_index, Some(142));
    assert_close(zs2.zg, 22.0);
    assert_close(zs2.zd, 5.5);
    assert_close(zs2.gg, 24.0);
    assert_close(zs2.dd, 1.0);
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
    segseg: Vec<serde_json::Value>,
    zs: Vec<GoldZs>,
    seg_zs: Vec<GoldZs>,
    bsp: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Stage1GoldMeta {
    merged_bars_count: usize,
    fx_count: usize,
    bi_count: usize,
    segment_count: usize,
    segseg_count: usize,
    zs_count: usize,
    seg_zs_count: usize,
    bsp_count: usize,
}

#[derive(Debug, Deserialize)]
struct GoldZs {
    index: usize,
    start_bi_index: Option<usize>,
    end_bi_index: Option<usize>,
    start_raw_index: Option<i64>,
    end_raw_index: Option<i64>,
    zg: f64,
    zd: f64,
    gg: f64,
    dd: f64,
}
