//! Stage-1 BSP construction aligned with local chan.py gold fixtures.
//!
//! Current scope:
//! - bi-level B1 / S1 from segment end breaking its last inner ZS.
//! - bi-level B1p / S1p from the chan.py treat_pz_bsp1 fallback path.
//! - bi-level B2 from the pullback after a stored B1 / S1.
//! - bi-level B2s from later same-side pullbacks while staying in the allowed segment context.
//! - segment-level B1 / S1 from segment end breaking segment-level ZS.
//!
//! Important compatibility detail:
//! chan.py exports BSP rows sorted by raw bar, but the `index` field is the generation index.
//! Therefore this module assigns bi-level indices before the final display sort, then appends
//! segment-level BSPs, and finally sorts rows without renumbering.
//!
//! Known gaps for later stages:
//! - T1P / 1p structural fallback is implemented for committed fixtures.
//! - T3A / 3a and T3B / 3b are implemented for structural BSP3 parity covered by committed fixtures.
//! - ChanConfig BSP options for enabled/types/follow_1/follow_2 are wired.
//! - Rate thresholds are wired for B2 and B2s.
//! - Exact chan.py MACD metric parity for T1P divergence is still a known gap.
//! - Divergence and peak filters are represented only by the behavior covered by current
//!   stage1 gold fixtures.

use super::config::{ChanBspConfig, ChanBspType};
use super::model::{ChanBi, ChanDirection, ChanSegment};
use super::zs::{ChanSegZs, ChanZs};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChanBsp {
    pub index: usize,
    pub bar_id: i64,
    pub price: f64,
    #[serde(rename = "type")]
    pub bs_type: String,
    pub level: String,
    pub bi_index: Option<usize>,
    pub segment_index: Option<usize>,
    pub confirmed: bool,
}

/// Build stage-1 BSP rows from Rust Chan structures.
///
/// The implementation intentionally follows the observable chan.py export behavior
/// from committed stage1 gold fixtures instead of trying to expose every chan.py
/// BSP configuration option at once.
pub fn build_bsp_with_config(
    bis: &[ChanBi],
    segments: &[ChanSegment],
    zs: &[ChanZs],
    seg_zs: &[ChanSegZs],
    config: &ChanBspConfig,
) -> Vec<ChanBsp> {
    if !config.enabled {
        return Vec::new();
    }

    let mut rows = build_bsp_core(bis, segments, zs, seg_zs, config);
    rows.retain(|row| output_type_enabled(config, row));
    rows
}

fn output_type_enabled(config: &ChanBspConfig, row: &ChanBsp) -> bool {
    match row.bs_type.as_str() {
        "B1" | "S1" => config.is_type_enabled(ChanBspType::T1),
        "B1p" | "S1p" => config.is_type_enabled(ChanBspType::T1p),
        "B2" | "S2" => config.is_type_enabled(ChanBspType::T2),
        "B2s" | "S2s" => config.is_type_enabled(ChanBspType::T2s),
        "B3a" | "S3a" => config.is_type_enabled(ChanBspType::T3a),
        "B3b" | "S3b" => config.is_type_enabled(ChanBspType::T3b),
        _ => false,
    }
}

pub fn build_bsp(
    bis: &[ChanBi],
    segments: &[ChanSegment],
    zs: &[ChanZs],
    seg_zs: &[ChanSegZs],
) -> Vec<ChanBsp> {
    build_bsp_core(bis, segments, zs, seg_zs, &ChanBspConfig::default())
}

fn build_bsp_core(
    bis: &[ChanBi],
    segments: &[ChanSegment],
    zs: &[ChanZs],
    seg_zs: &[ChanSegZs],
    config: &ChanBspConfig,
) -> Vec<ChanBsp> {
    let mut bi_rows = Vec::new();
    let bi_seg = bi_segment_map(bis.len(), segments);

    for (seg_pos, seg) in segments.iter().enumerate() {
        let Some(end_i) = seg.end_parent_index else {
            continue;
        };
        if end_i >= bis.len() {
            continue;
        }

        let seg_zs_count = zs
            .iter()
            .filter(|z| z.parent_segment_index == Some(seg.index))
            .count();

        let Some(z) = zs
            .iter()
            .filter(|z| z.parent_segment_index == Some(seg.index))
            .max_by_key(|z| z.index)
        else {
            if let Some(point) = maybe_t1p_bsp(seg, bis, &bi_seg, config, seg_zs_count) {
                bi_rows.push(point);
            }
            continue;
        };

        let bi = &bis[end_i];
        let has_bsp1 = breaks(bi.direction, bi.end_price, z.zd, z.zg);
        if has_bsp1 {
            bi_rows.push(bi_bsp(bi, "1"));
        } else if config.bsp2_follow_1 {
            continue;
        }

        if let Some(point) = maybe_t3a_bsp(
            seg,
            segments.get(seg_pos + 1),
            segments,
            bis,
            &bi_seg,
            zs,
            config,
            has_bsp1,
        ) {
            bi_rows.push(point);
        }

        if let Some(point) = maybe_t3b_bsp(
            seg,
            segments.get(seg_pos + 1),
            segments.len(),
            bis,
            &bi_seg,
            z,
            config,
            has_bsp1,
        ) {
            bi_rows.push(point);
        }

        if end_i + 2 >= bis.len() {
            continue;
        }

        let break_bi = &bis[end_i + 1];
        let b2 = &bis[end_i + 2];
        let b2_seg = bi_seg.get(b2.index).copied().flatten();

        let has_bsp2 = amp(b2) / amp(break_bi) <= config.max_bsp2_rate;
        if has_bsp2 {
            bi_rows.push(bi_bsp(b2, "2"));
        } else if config.bsp2s_follow_2 {
            continue;
        }

        let mut j = end_i + 4;
        while j < bis.len() {
            let cand = &bis[j];

            if let Some(base_seg) = b2_seg {
                let Some(cand_seg) = bi_seg.get(cand.index).copied().flatten() else {
                    break;
                };

                if cand_seg != base_seg {
                    let base_confirmed = segments
                        .get(base_seg)
                        .map(|segment| segment.confirmed)
                        .unwrap_or(false);

                    if cand_seg < segments.len().saturating_sub(1)
                        || cand_seg.saturating_sub(base_seg) >= 2
                        || base_confirmed
                    {
                        break;
                    }
                }
            }

            if !overlap(range(b2), range(cand)) {
                break;
            }
            if breaks_break(cand, break_bi) {
                break;
            }
            if (cand.end_price - break_bi.end_price).abs() / amp(break_bi) > config.max_bsp2s_rate {
                break;
            }

            bi_rows.push(bi_bsp(cand, "2s"));
            j += 2;
        }
    }

    bi_rows.sort_by(|a, b| {
        a.bi_index
            .cmp(&b.bi_index)
            .then_with(|| a.bar_id.cmp(&b.bar_id))
            .then_with(|| a.bs_type.cmp(&b.bs_type))
    });

    for (index, row) in bi_rows.iter_mut().enumerate() {
        row.index = index;
    }

    let mut rows = bi_rows;

    for seg in segments {
        for z in seg_zs {
            if seg.index > z.end_segment_index && breaks(seg.direction, seg.end_price, z.zd, z.zg) {
                push_seg_bsp(&mut rows, seg, "1");
                break;
            }
        }
    }

    rows.sort_by(|a, b| {
        a.bar_id
            .cmp(&b.bar_id)
            .then_with(|| a.bs_type.cmp(&b.bs_type))
            .then_with(|| a.level.cmp(&b.level))
    });

    rows
}

fn maybe_t3a_bsp(
    seg: &ChanSegment,
    next_seg: Option<&ChanSegment>,
    segments: &[ChanSegment],
    bis: &[ChanBi],
    bi_seg: &[Option<usize>],
    zs: &[ChanZs],
    config: &ChanBspConfig,
    has_bsp1: bool,
) -> Option<ChanBsp> {
    if !config.is_type_enabled(ChanBspType::T3a) {
        return None;
    }
    if config.bsp3_follow_1 && !has_bsp1 {
        return None;
    }

    let bsp1_i = seg.end_parent_index?;
    let next_seg = next_seg?;
    let next_seg_idx = next_seg.index;

    let mut next_zs: Vec<&ChanZs> = zs
        .iter()
        .filter(|z| z.parent_segment_index == Some(next_seg_idx))
        .collect();
    next_zs.sort_by_key(|z| z.index);

    let first_zs = next_zs.first().copied()?;
    if config.strict_bsp3 && first_zs.start_bi_index != bsp1_i.saturating_add(2) {
        return None;
    }

    for (zs_idx, z) in next_zs.into_iter().enumerate() {
        if zs_idx >= config.bsp3a_max_zs_cnt {
            break;
        }

        // chan.py uses zs.bi_out.idx + 1. In normalized Rust ZS, end_bi_index
        // is the last in-ZS BI, while chan.py bi_out is the following BI.
        let cand_i = z.end_bi_index.saturating_add(2);
        if cand_i >= bis.len() {
            break;
        }

        let cand = &bis[cand_i];
        let cand_seg_idx = bi_seg.get(cand_i).copied().flatten();

        match cand_seg_idx {
            None => {
                if next_seg_idx != segments.len().saturating_sub(1) {
                    break;
                }
            }
            Some(seg_idx) if seg_idx != next_seg_idx => {
                let parent_len = segments
                    .get(seg_idx)
                    .and_then(|segment| {
                        Some(segment.end_parent_index? - segment.start_parent_index? + 1)
                    })
                    .unwrap_or(usize::MAX);
                if parent_len >= 3 {
                    break;
                }
            }
            _ => {}
        }

        if cand.direction == next_seg.direction {
            break;
        }

        if cand_seg_idx != Some(next_seg_idx) && next_seg_idx < segments.len().saturating_sub(2) {
            break;
        }

        if bsp3_back2zs(cand, z) {
            continue;
        }

        if config.bsp3_peak && !bsp3_break_zspeak(cand, z) {
            continue;
        }

        return Some(bi_bsp(cand, "3a"));
    }

    None
}

fn bsp3_break_zspeak(bi: &ChanBi, zs: &ChanZs) -> bool {
    match bi.direction {
        ChanDirection::Down => bi_high(bi) >= zs.gg,
        ChanDirection::Up => bi_low(bi) <= zs.dd,
        ChanDirection::Unknown => false,
    }
}

fn maybe_t3b_bsp(
    seg: &ChanSegment,
    next_seg: Option<&ChanSegment>,
    segments_len: usize,
    bis: &[ChanBi],
    bi_seg: &[Option<usize>],
    cmp_zs: &ChanZs,
    config: &ChanBspConfig,
    has_bsp1: bool,
) -> Option<ChanBsp> {
    if !config.is_type_enabled(ChanBspType::T3b) {
        return None;
    }
    if config.bsp3_follow_1 && !has_bsp1 {
        return None;
    }

    let bsp1_i = seg.end_parent_index?;
    if bsp1_i >= bis.len() {
        return None;
    }

    // chan.py strict mode checks cmp_zs.bi_out == bsp1_bi.
    // Rust ChanZs stores the last in-ZS BI as end_bi_index, so bi_out maps to end_bi_index + 1.
    if config.strict_bsp3 && cmp_zs.end_bi_index.saturating_add(1) != bsp1_i {
        return None;
    }

    let next_seg = next_seg?;
    let next_seg_idx = next_seg.index;
    let end_bi_idx = next_seg.end_parent_index?;

    let mut cand_i = bsp1_i.saturating_add(2);
    while cand_i < bis.len() {
        if cand_i > end_bi_idx {
            break;
        }

        let cand_seg_idx = bi_seg.get(cand_i).copied().flatten();
        if let Some(seg_idx) = cand_seg_idx {
            if seg_idx != next_seg_idx && seg_idx < segments_len.saturating_sub(1) {
                break;
            }
        }

        let cand = &bis[cand_i];
        if bsp3_back2zs(cand, cmp_zs) {
            cand_i = cand_i.saturating_add(2);
            continue;
        }

        return Some(bi_bsp(cand, "3b"));
    }

    None
}

fn bsp3_back2zs(bi: &ChanBi, zs: &ChanZs) -> bool {
    match bi.direction {
        ChanDirection::Down => bi_low(bi) < zs.zg,
        ChanDirection::Up => bi_high(bi) > zs.zd,
        ChanDirection::Unknown => false,
    }
}

fn maybe_t1p_bsp(
    seg: &ChanSegment,
    bis: &[ChanBi],
    bi_seg: &[Option<usize>],
    config: &ChanBspConfig,
    seg_zs_count: usize,
) -> Option<ChanBsp> {
    if !config.is_type_enabled(ChanBspType::T1p) {
        return None;
    }
    if seg_zs_count < config.min_zs_cnt_for_t1p {
        return None;
    }

    let end_i = seg.end_parent_index?;
    if end_i >= bis.len() || end_i < 2 {
        return None;
    }

    let last_bi = &bis[end_i];
    let pre_bi = &bis[end_i - 2];

    if last_bi.direction != seg.direction {
        return None;
    }

    let last_seg = bi_seg.get(last_bi.index).copied().flatten();
    let pre_seg = bi_seg.get(pre_bi.index).copied().flatten();
    if last_seg != Some(seg.index) || pre_seg != Some(seg.index) {
        return None;
    }

    match last_bi.direction {
        ChanDirection::Down => {
            if bi_low(last_bi) > bi_low(pre_bi) {
                return None;
            }
        }
        ChanDirection::Up => {
            if bi_high(last_bi) < bi_high(pre_bi) {
                return None;
            }
        }
        ChanDirection::Unknown => return None,
    }

    Some(bi_bsp(last_bi, "1p"))
}

fn push_seg_bsp(rows: &mut Vec<ChanBsp>, seg: &ChanSegment, t: &str) {
    let mut point = seg_bsp(seg, t);
    point.index = rows.len();
    rows.push(point);
}

fn bi_bsp(bi: &ChanBi, t: &str) -> ChanBsp {
    let is_buy = bi.direction == ChanDirection::Down;
    ChanBsp {
        index: 0,
        bar_id: bi.end_bar_id,
        price: bi.end_price,
        bs_type: format!("{}{}", if is_buy { "B" } else { "S" }, t),
        level: "bi".to_string(),
        bi_index: Some(bi.index),
        segment_index: None,
        confirmed: true,
    }
}

fn seg_bsp(seg: &ChanSegment, t: &str) -> ChanBsp {
    let is_buy = seg.direction == ChanDirection::Down;
    ChanBsp {
        index: 0,
        bar_id: seg.end_bar_id,
        price: seg.end_price,
        bs_type: format!("{}{}", if is_buy { "B" } else { "S" }, t),
        level: "seg".to_string(),
        bi_index: None,
        segment_index: Some(seg.index),
        confirmed: true,
    }
}

fn bi_segment_map(bi_len: usize, segments: &[ChanSegment]) -> Vec<Option<usize>> {
    let mut result = vec![None; bi_len];

    for segment in segments {
        let Some(start) = segment.start_parent_index else {
            continue;
        };
        let Some(end) = segment.end_parent_index else {
            continue;
        };

        for bi_index in start..=end.min(bi_len.saturating_sub(1)) {
            result[bi_index] = Some(segment.index);
        }
    }

    result
}

fn breaks(direction: ChanDirection, price: f64, zd: f64, zg: f64) -> bool {
    match direction {
        ChanDirection::Down => price < zd,
        ChanDirection::Up => price > zg,
        ChanDirection::Unknown => false,
    }
}

fn range(bi: &ChanBi) -> (f64, f64) {
    (bi_low(bi), bi_high(bi))
}

fn bi_low(bi: &ChanBi) -> f64 {
    bi.start_price.min(bi.end_price)
}

fn bi_high(bi: &ChanBi) -> f64 {
    bi.start_price.max(bi.end_price)
}

fn overlap(a: (f64, f64), b: (f64, f64)) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}

fn amp(bi: &ChanBi) -> f64 {
    (bi.end_price - bi.start_price).abs()
}

fn breaks_break(candidate: &ChanBi, break_bi: &ChanBi) -> bool {
    let c = range(candidate);
    let b = range(break_bi);
    (candidate.direction == ChanDirection::Down && c.0 < b.0)
        || (candidate.direction == ChanDirection::Up && c.1 > b.1)
}
