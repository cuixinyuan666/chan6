//! Stage-1 BSP construction aligned with local chan.py gold fixtures.
//!
//! Current scope:
//! - bi-level B1 / S1 from segment end breaking its last inner ZS.
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
//! - T1P / 1p is not implemented.
//! - T3A / 3a and T3B / 3b are not implemented.
//! - ChanConfig BSP options such as target bs_type, follow_1, follow_2, and rate thresholds
//!   are not wired into Rust yet.
//! - Divergence and peak filters are represented only by the behavior covered by current
//!   stage1 gold fixtures.

use super::config::ChanBspConfig;
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

    build_bsp(bis, segments, zs, seg_zs)
}

pub fn build_bsp(
    bis: &[ChanBi],
    segments: &[ChanSegment],
    zs: &[ChanZs],
    seg_zs: &[ChanSegZs],
) -> Vec<ChanBsp> {
    let mut bi_rows = Vec::new();
    let bi_seg = bi_segment_map(bis.len(), segments);

    for seg in segments {
        let Some(end_i) = seg.end_parent_index else {
            continue;
        };
        if end_i >= bis.len() {
            continue;
        }

        let Some(z) = zs
            .iter()
            .filter(|z| z.parent_segment_index == Some(seg.index))
            .max_by_key(|z| z.index)
        else {
            continue;
        };

        let bi = &bis[end_i];
        if !breaks(bi.direction, bi.end_price, z.zd, z.zg) {
            continue;
        }

        bi_rows.push(bi_bsp(bi, "1"));

        if end_i + 2 >= bis.len() {
            continue;
        }

        let break_bi = &bis[end_i + 1];
        let b2 = &bis[end_i + 2];
        let b2_seg = bi_seg.get(b2.index).copied().flatten();

        if amp(b2) / amp(break_bi) > 0.9999 {
            continue;
        }

        bi_rows.push(bi_bsp(b2, "2"));

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
            if (cand.end_price - break_bi.end_price).abs() / amp(break_bi) > 0.9999 {
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
    (
        bi.start_price.min(bi.end_price),
        bi.start_price.max(bi.end_price),
    )
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
