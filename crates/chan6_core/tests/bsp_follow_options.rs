use chan6_core::chan::bsp::build_bsp_with_config;
use chan6_core::chan::config::{ChanBspConfig, ChanBspType};
use chan6_core::chan::model::{ChanBi, ChanDirection, ChanSegment};
use chan6_core::chan::zs::ChanZs;

#[test]
fn bsp2_follow_1_blocks_b2_without_real_bsp1_by_default() {
    let bis = vec![
        bi(0, ChanDirection::Up, 8.0, 18.0),
        bi(1, ChanDirection::Down, 18.0, 11.0),
        bi(2, ChanDirection::Down, 20.0, 12.0),
        bi(3, ChanDirection::Up, 12.0, 22.0),
        bi(4, ChanDirection::Down, 22.0, 14.0),
    ];
    let segments = vec![segment(0, ChanDirection::Down, 0, 2, 20.0, 12.0)];
    let zs = vec![zs(0, Some(0), 0, 2, 15.0, 10.0)];

    let default_rows = build_bsp_with_config(&bis, &segments, &zs, &[], &ChanBspConfig::default());
    assert!(default_rows.is_empty());

    let mut config = ChanBspConfig::default();
    config.bsp2_follow_1 = false;
    config.types = vec![ChanBspType::T2];

    let loose_rows = build_bsp_with_config(&bis, &segments, &zs, &[], &config);
    assert_eq!(loose_rows.len(), 1);
    assert_eq!(loose_rows[0].bs_type, "B2");
    assert_eq!(loose_rows[0].bi_index, Some(4));
}

#[test]
fn bsp2s_follow_2_blocks_b2s_when_b2_retrace_fails_by_default() {
    let bis = vec![
        bi(0, ChanDirection::Up, 8.0, 18.0),
        bi(1, ChanDirection::Down, 18.0, 11.0),
        bi(2, ChanDirection::Down, 20.0, 8.0),
        bi(3, ChanDirection::Up, 12.0, 22.0),
        bi(4, ChanDirection::Down, 22.0, 11.0),
        bi(5, ChanDirection::Up, 11.0, 21.0),
        bi(6, ChanDirection::Down, 21.0, 14.0),
    ];
    let segments = vec![segment(0, ChanDirection::Down, 0, 2, 20.0, 8.0)];
    let zs = vec![zs(0, Some(0), 0, 2, 15.0, 10.0)];

    let mut default_config = ChanBspConfig::default();
    default_config.types = vec![ChanBspType::T2s];

    let default_rows = build_bsp_with_config(&bis, &segments, &zs, &[], &default_config);
    assert!(default_rows.is_empty());

    let mut loose_config = default_config;
    loose_config.bsp2s_follow_2 = false;

    let loose_rows = build_bsp_with_config(&bis, &segments, &zs, &[], &loose_config);
    assert_eq!(loose_rows.len(), 1);
    assert_eq!(loose_rows[0].bs_type, "B2s");
    assert_eq!(loose_rows[0].bi_index, Some(6));
}

fn bi(index: usize, direction: ChanDirection, start_price: f64, end_price: f64) -> ChanBi {
    ChanBi {
        index,
        direction,
        start_fx_index: index,
        end_fx_index: index + 1,
        start_bar_id: index as i64 * 10,
        start_price,
        end_bar_id: index as i64 * 10 + 5,
        end_price,
        confirmed: true,
        prev_index: index.checked_sub(1),
        next_index: Some(index + 1),
        parent_segment_index: None,
    }
}

fn segment(
    index: usize,
    direction: ChanDirection,
    start_parent_index: usize,
    end_parent_index: usize,
    start_price: f64,
    end_price: f64,
) -> ChanSegment {
    ChanSegment {
        n: 1,
        input_n: None,
        index,
        direction,
        start_parent_index: Some(start_parent_index),
        end_parent_index: Some(end_parent_index),
        start_bar_id: start_parent_index as i64 * 10,
        start_price,
        end_bar_id: end_parent_index as i64 * 10 + 5,
        end_price,
        confirmed: false,
        reason: "test_segment".to_string(),
    }
}

fn zs(
    index: usize,
    parent_segment_index: Option<usize>,
    start_bi_index: usize,
    end_bi_index: usize,
    zg: f64,
    zd: f64,
) -> ChanZs {
    ChanZs {
        index,
        start_bi_index,
        end_bi_index,
        start_bar_id: start_bi_index as i64 * 10,
        end_bar_id: end_bi_index as i64 * 10 + 5,
        zg,
        zd,
        gg: zg,
        dd: zd,
        confirmed: true,
        parent_segment_index,
    }
}
