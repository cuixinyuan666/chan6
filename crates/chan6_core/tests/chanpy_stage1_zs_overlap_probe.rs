use chan6_core::chan::analyze_chan_basic;
use chan6_core::chan::model::{ChanDirection, CHAN_SEGMENT_N_LINE};
use chan6_core::model::KLine1m;
use serde::Deserialize;

#[test]
fn chanpy_stage1_zs_overlap_probe_exposes_zs_seg_zs_and_bsp_gold() {
    let symbol = "stage1_zs_overlap_probe_candidate";
    let csv =
        include_str!("../../../fixtures/chanpy_stage1/input/stage1_zs_overlap_probe_candidate.csv");
    let gold_text = include_str!(
        "../../../fixtures/chanpy_stage1/gold/stage1_zs_overlap_probe_candidate_chanpy_gold.json"
    );

    let gold: Stage1Gold = serde_json::from_str(gold_text).unwrap();
    let klines = parse_stage1_csv(symbol, csv);
    let snapshot = analyze_chan_basic(&klines);

    // Stage-1 Rust already aligns hichan merged/fx/bi/seg on this fixture.
    assert_eq!(snapshot.meta.merged_count, gold.meta.merged_bars_count);
    assert_eq!(snapshot.meta.fx_count, gold.meta.fx_count);
    assert_eq!(snapshot.meta.bi_count, gold.meta.bi_count);
    assert_eq!(snapshot.meta.segment_count, gold.meta.segment_count);

    // This fixture is the first hichan gold sample that exposes higher structures.
    // Rust does not implement zs/seg_zs/bsp yet; this locks the authoritative gold
    // output so the next implementation stage can be driven by executable data.
    assert_eq!(gold.meta.segseg_count, 0);
    assert_eq!(gold.meta.zs_count, 3);
    assert_eq!(gold.meta.seg_zs_count, 1);
    assert_eq!(gold.meta.bsp_count, 7);

    assert_eq!(gold.segseg.len(), gold.meta.segseg_count);
    assert_eq!(gold.zs.len(), gold.meta.zs_count);
    assert_eq!(gold.seg_zs.len(), gold.meta.seg_zs_count);
    assert_eq!(gold.bsp.len(), gold.meta.bsp_count);

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

    assert_eq!(gold.zs[0].index, 0);
    assert_eq!(gold.zs[0].start_bi_index, Some(4));
    assert_eq!(gold.zs[0].end_bi_index, Some(8));
    assert_eq!(gold.zs[0].start_raw_index, Some(17));
    assert_eq!(gold.zs[0].end_raw_index, Some(37));
    assert_close(gold.zs[0].zg, 13.5);
    assert_close(gold.zs[0].zd, 5.8);
    assert_close(gold.zs[0].gg, 14.2);
    assert_close(gold.zs[0].dd, 3.0);

    assert_eq!(gold.seg_zs[0].index, 0);
    assert_eq!(gold.seg_zs[0].start_raw_index, Some(13));
    assert_eq!(gold.seg_zs[0].end_raw_index, Some(116));
    assert_close(gold.seg_zs[0].zg, 15.0);
    assert_close(gold.seg_zs[0].zd, 2.5);
    assert_close(gold.seg_zs[0].gg, 18.5);
    assert_close(gold.seg_zs[0].dd, 0.5);

    assert_eq!(gold.bsp[0].raw_index, 41);
    assert_eq!(gold.bsp[0].price, 2.5);
    assert_eq!(gold.bsp[0].kind, "B1");
    assert_eq!(gold.bsp[0].level, "bi");
    assert_eq!(gold.bsp[0].bi_index, Some(9));
    assert_eq!(gold.bsp[0].confirmed, true);

    let last_bsp = gold.bsp.last().unwrap();
    assert_eq!(last_bsp.raw_index, 156);
    assert_eq!(last_bsp.kind, "S1");
    assert_eq!(last_bsp.level, "seg");
    assert_eq!(last_bsp.seg_index, Some(6));
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
    seg: Vec<GoldSegment>,
    segseg: Vec<GoldSegment>,
    zs: Vec<GoldZs>,
    seg_zs: Vec<GoldZs>,
    bsp: Vec<GoldBsp>,
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
struct GoldSegment {
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

#[derive(Debug, Deserialize)]
struct GoldBsp {
    raw_index: i64,
    price: f64,
    #[serde(rename = "type")]
    kind: String,
    level: String,
    bi_index: Option<usize>,
    seg_index: Option<usize>,
    confirmed: bool,
}
