use chan6_core::chan::analyze_chan_basic_with_config;
use chan6_core::chan::config::{ChanBspType, ChanConfig};
use chan6_core::model::KLine1m;
use serde_json::Value;

#[test]
fn chanpy_stage1_t3b_probe_emits_b3b_like_chanpy_gold() {
    let symbol = "stage1_t3b_probe_candidate";
    let csv = include_str!("../../../fixtures/chanpy_stage1/input/stage1_t3b_probe_candidate.csv");
    let gold_text = include_str!(
        "../../../fixtures/chanpy_stage1/gold/stage1_t3b_probe_candidate_chanpy_gold.json"
    );

    let gold: Value = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(symbol, csv);

    let mut config = ChanConfig::default();
    config.bsp.types = vec![ChanBspType::T1, ChanBspType::T3b];

    let snapshot = analyze_chan_basic_with_config(&klines, "1m", &config);
    let expected = gold["bsp"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["type"].as_str() == Some("B3b"))
        .expect("gold should contain B3b");

    let actual_types: Vec<_> = snapshot
        .bsp
        .iter()
        .map(|row| row.bs_type.as_str())
        .collect();
    let actual = snapshot
        .bsp
        .iter()
        .find(|row| row.bs_type == "B3b")
        .unwrap_or_else(|| panic!("missing B3b; actual BSP types: {:?}", actual_types));

    assert_eq!(actual.bar_id, expected["raw_index"].as_i64().unwrap());
    assert_close(actual.price, expected["price"].as_f64().unwrap());
    assert_eq!(actual.level, expected["level"].as_str().unwrap());
    assert_eq!(
        actual.bi_index,
        expected["bi_index"].as_u64().map(|x| x as usize)
    );
    assert_eq!(
        actual.segment_index,
        expected["seg_index"].as_u64().map(|x| x as usize)
    );
    assert_eq!(actual.confirmed, expected["confirmed"].as_bool().unwrap());

    assert_eq!(actual.bs_type, "B3b");
    assert_eq!(actual.bar_id, 142);
    assert_close(actual.price, 16.05);
    assert_eq!(actual.bi_index, Some(27));
}

#[test]
fn chanpy_stage1_t3b_probe_type_filter_can_exclude_t3b() {
    let symbol = "stage1_t3b_probe_candidate";
    let csv = include_str!("../../../fixtures/chanpy_stage1/input/stage1_t3b_probe_candidate.csv");
    let klines = parse_stage1_csv(symbol, csv);

    let mut config = ChanConfig::default();
    config.bsp.types = vec![ChanBspType::T1];

    let snapshot = analyze_chan_basic_with_config(&klines, "1m", &config);
    assert!(!snapshot.bsp.iter().any(|row| row.bs_type == "B3b"));
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
    assert!(
        (actual - expected).abs() < 1e-9,
        "actual={actual}, expected={expected}"
    );
}
