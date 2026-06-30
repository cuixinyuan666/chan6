use chan6_core::chan::config::{ChanBspType, ChanConfig};
use chan6_core::chan::{analyze_chan_basic, analyze_chan_basic_with_config};
use chan6_core::model::KLine1m;
use serde_json::Value;

#[test]
fn chanpy_stage1_t1p_probe_matches_chanpy_gold_when_min_zs_cnt_is_zero() {
    let symbol = "stage1_t1p_probe_candidate";
    let csv = include_str!("../../../fixtures/chanpy_stage1/input/stage1_t1p_probe_candidate.csv");
    let gold_text = include_str!("../../../fixtures/chanpy_stage1/gold/stage1_t1p_probe_candidate_chanpy_gold.json");

    let gold: Value = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(symbol, csv);

    let default_snapshot = analyze_chan_basic(&klines);
    assert!(default_snapshot.bsp.is_empty());

    let mut config = ChanConfig::default();
    config.bsp.types = vec![ChanBspType::T1p];
    config.bsp.min_zs_cnt_for_t1p = 0;

    let snapshot = analyze_chan_basic_with_config(&klines, "1m", &config);
    let meta = &gold["meta"];
    let bsp = gold["bsp"].as_array().unwrap();

    assert_eq!(snapshot.meta.merged_count, meta["merged_bars_count"].as_u64().unwrap() as usize);
    assert_eq!(snapshot.meta.fx_count, meta["fx_count"].as_u64().unwrap() as usize);
    assert_eq!(snapshot.meta.bi_count, meta["bi_count"].as_u64().unwrap() as usize);
    assert_eq!(snapshot.meta.segment_count, meta["segment_count"].as_u64().unwrap() as usize);
    assert_eq!(snapshot.zs.len(), meta["zs_count"].as_u64().unwrap() as usize);
    assert_eq!(snapshot.bsp.len(), bsp.len());
    assert_eq!(meta["bsp_count"].as_u64().unwrap(), 1);

    let actual = &snapshot.bsp[0];
    let expected = &bsp[0];

    assert_eq!(actual.index, expected["index"].as_u64().unwrap() as usize);
    assert_eq!(actual.bar_id, expected["raw_index"].as_i64().unwrap());
    assert_close(actual.price, expected["price"].as_f64().unwrap());
    assert_eq!(actual.bs_type, expected["type"].as_str().unwrap());
    assert_eq!(actual.level, expected["level"].as_str().unwrap());
    assert_eq!(actual.bi_index, expected["bi_index"].as_u64().map(|x| x as usize));
    assert_eq!(actual.segment_index, expected["seg_index"].as_u64().map(|x| x as usize));
    assert_eq!(actual.confirmed, expected["confirmed"].as_bool().unwrap());

    assert_eq!(actual.bs_type, "S1p");
    assert_eq!(actual.bar_id, 13);
    assert_close(actual.price, 15.0);
    assert_eq!(actual.bi_index, Some(2));
}

#[test]
fn chanpy_stage1_t1p_probe_type_filter_can_exclude_t1p() {
    let symbol = "stage1_t1p_probe_candidate";
    let csv = include_str!("../../../fixtures/chanpy_stage1/input/stage1_t1p_probe_candidate.csv");
    let klines = parse_stage1_csv(symbol, csv);

    let mut config = ChanConfig::default();
    config.bsp.types = vec![ChanBspType::T1];
    config.bsp.min_zs_cnt_for_t1p = 0;

    let snapshot = analyze_chan_basic_with_config(&klines, "1m", &config);
    assert!(snapshot.bsp.is_empty());
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

            let cols: Vec<_> = line.split(",").collect();
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
    assert!((actual - expected).abs() < 1e-9, "actual={actual}, expected={expected}");
}
