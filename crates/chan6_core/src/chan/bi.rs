use super::model::{ChanBi, ChanDirection, ChanFx, ChanFxKind, ChanMergedBar};

/// Minimum distance between two FX merged indexes for first-stage BI construction.
///
/// This is intentionally aligned to chan.py strict BI behavior for the stage-1
/// fixtures: adjacent top/bottom FX are not enough to form a BI.
pub const DEFAULT_MIN_BI_MERGED_SPAN: usize = 4;

pub fn build_bis(fxs: &[ChanFx]) -> Vec<ChanBi> {
    build_bis_with_min_span(fxs, DEFAULT_MIN_BI_MERGED_SPAN)
}

pub fn build_bis_with_min_span(fxs: &[ChanFx], min_merged_span: usize) -> Vec<ChanBi> {
    build_bis_internal(fxs, min_merged_span, None)
}

pub fn build_bis_with_merged_bars(fxs: &[ChanFx], merged_bars: &[ChanMergedBar]) -> Vec<ChanBi> {
    build_bis_internal(fxs, DEFAULT_MIN_BI_MERGED_SPAN, Some(merged_bars))
}

fn build_bis_internal(
    fxs: &[ChanFx],
    min_merged_span: usize,
    merged_bars: Option<&[ChanMergedBar]>,
) -> Vec<ChanBi> {
    let mut bis = Vec::new();
    let mut free_fxs: Vec<&ChanFx> = Vec::new();
    let mut last_bi_end: Option<&ChanFx> = None;
    let mut pending_opposite_fx: Option<&ChanFx> = None;

    for fx in fxs {
        if let Some(last_end) = last_bi_end {
            if fx.kind == last_end.kind {
                if is_stronger_same_kind_fx(fx, last_end) {
                    update_last_bi_end(&mut bis, fx);
                    last_bi_end = Some(fx);
                    pending_opposite_fx = None;
                }
                continue;
            }

            if let Some(pending) = pending_opposite_fx {
                if !is_stronger_or_equal_same_kind_fx(fx, pending) {
                    continue;
                }
            }

            pending_opposite_fx = Some(fx);
            let candidate = pending_opposite_fx.expect("pending opposite fx just set");

            if can_make_bi(last_end, candidate, min_merged_span, merged_bars) {
                push_bi(&mut bis, last_end, candidate);
                last_bi_end = Some(candidate);
                pending_opposite_fx = None;
            }
            continue;
        }

        let mut first_valid_start = None;
        for existing in &free_fxs {
            if existing.kind == fx.kind {
                continue;
            }
            if can_make_bi(existing, fx, min_merged_span, merged_bars) {
                first_valid_start = Some(*existing);
                break;
            }
        }

        if let Some(start) = first_valid_start {
            push_bi(&mut bis, start, fx);
            last_bi_end = Some(fx);
        } else {
            free_fxs.push(fx);
        }
    }

    if let Some(merged_bars) = merged_bars {
        append_active_tail_bi(&mut bis, fxs, merged_bars, min_merged_span);
    }

    link_bis(&mut bis);
    bis
}

fn can_make_bi(
    start: &ChanFx,
    end: &ChanFx,
    min_merged_span: usize,
    merged_bars: Option<&[ChanMergedBar]>,
) -> bool {
    fx_span_is_enough(start, end, min_merged_span) && check_fx_valid(start, end, merged_bars)
}

fn check_fx_valid(start: &ChanFx, end: &ChanFx, merged_bars: Option<&[ChanMergedBar]>) -> bool {
    let Some(merged_bars) = merged_bars else {
        return true;
    };

    match (start.kind, end.kind) {
        (ChanFxKind::Top, ChanFxKind::Bottom) => {
            let Some(end_high) = neighborhood_calc_high(end, merged_bars) else {
                return true;
            };
            let Some(start_low) = neighborhood_calc_low(start, merged_bars) else {
                return true;
            };
            start.price > end_high && end.price < start_low
        }
        (ChanFxKind::Bottom, ChanFxKind::Top) => {
            let Some(end_low) = neighborhood_calc_low(end, merged_bars) else {
                return true;
            };
            let Some(start_high) = neighborhood_calc_high(start, merged_bars) else {
                return true;
            };
            start.price < end_low && end.price > start_high
        }
        _ => false,
    }
}

fn neighborhood_calc_high(fx: &ChanFx, merged_bars: &[ChanMergedBar]) -> Option<f64> {
    let left = merged_bars.get(fx.left_merged_index)?.calc_high;
    let center = merged_bars.get(fx.center_merged_index)?.calc_high;
    let right = merged_bars.get(fx.right_merged_index)?.calc_high;
    Some(left.max(center).max(right))
}

fn neighborhood_calc_low(fx: &ChanFx, merged_bars: &[ChanMergedBar]) -> Option<f64> {
    let left = merged_bars.get(fx.left_merged_index)?.calc_low;
    let center = merged_bars.get(fx.center_merged_index)?.calc_low;
    let right = merged_bars.get(fx.right_merged_index)?.calc_low;
    Some(left.min(center).min(right))
}

fn append_active_tail_bi(
    bis: &mut Vec<ChanBi>,
    fxs: &[ChanFx],
    merged_bars: &[ChanMergedBar],
    min_merged_span: usize,
) {
    let Some(last_bi) = bis.last() else {
        return;
    };
    let Some(start_fx) = fxs.get(last_bi.end_fx_index) else {
        return;
    };

    let Some((end_merged_index, end_bar_id, end_price, direction)) =
        active_tail_endpoint(start_fx, merged_bars)
    else {
        return;
    };

    if end_merged_index.saturating_sub(start_fx.merged_index) < min_merged_span {
        return;
    }
    if end_bar_id == last_bi.end_bar_id {
        return;
    }

    bis.push(ChanBi {
        index: bis.len(),
        direction,
        start_fx_index: start_fx.index,
        end_fx_index: fxs.len(),
        start_bar_id: start_fx.bar_id,
        start_price: start_fx.price,
        end_bar_id,
        end_price,
        confirmed: false,
        prev_index: None,
        next_index: None,
        parent_segment_index: None,
    });
}

fn active_tail_endpoint(
    start_fx: &ChanFx,
    merged_bars: &[ChanMergedBar],
) -> Option<(usize, i64, f64, ChanDirection)> {
    let start = start_fx.merged_index.saturating_add(1);
    if start >= merged_bars.len() {
        return None;
    }

    match start_fx.kind {
        ChanFxKind::Top => merged_bars[start..]
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.calc_low.total_cmp(&right.calc_low))
            .and_then(|(offset, merged)| {
                if merged.calc_low < start_fx.price {
                    Some((
                        start + offset,
                        merged.calc_low_bar_id,
                        merged.calc_low,
                        ChanDirection::Down,
                    ))
                } else {
                    None
                }
            }),
        ChanFxKind::Bottom => merged_bars[start..]
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.calc_high.total_cmp(&right.calc_high))
            .and_then(|(offset, merged)| {
                if merged.calc_high > start_fx.price {
                    Some((
                        start + offset,
                        merged.calc_high_bar_id,
                        merged.calc_high,
                        ChanDirection::Up,
                    ))
                } else {
                    None
                }
            }),
    }
}

fn update_last_bi_end(bis: &mut [ChanBi], end: &ChanFx) {
    let Some(last_bi) = bis.last_mut() else {
        return;
    };

    last_bi.end_fx_index = end.index;
    last_bi.end_bar_id = end.bar_id;
    last_bi.end_price = end.price;
    last_bi.confirmed = end.confirmed;
}

fn push_bi(bis: &mut Vec<ChanBi>, start: &ChanFx, end: &ChanFx) {
    bis.push(ChanBi::new(
        bis.len(),
        start.index,
        end.index,
        start.bar_id,
        start.price,
        end.bar_id,
        end.price,
    ));
}

/// Legacy helper for unit-level same-kind strength checks.
///
/// Full BI construction should use `build_bis_with_merged_bars` when merged bar
/// context is available, because chan.py's first-BI free candidate and
/// check_fx_valid flow is not equivalent to blindly normalizing all FX before BI.
pub fn normalize_fxs_for_bi(fxs: &[ChanFx], min_merged_span: usize) -> Vec<&ChanFx> {
    let mut normalized: Vec<&ChanFx> = Vec::new();

    for fx in fxs {
        let Some(last) = normalized.last().copied() else {
            normalized.push(fx);
            continue;
        };

        if fx.kind == last.kind {
            if is_stronger_same_kind_fx(fx, last) {
                normalized.pop();
                normalized.push(fx);
            }
            continue;
        }

        if fx_span_is_enough(last, fx, min_merged_span) {
            normalized.push(fx);
        }
    }

    normalized
}

fn is_stronger_or_equal_same_kind_fx(candidate: &ChanFx, current: &ChanFx) -> bool {
    if candidate.kind != current.kind {
        return false;
    }

    match candidate.kind {
        ChanFxKind::Top => candidate.price >= current.price,
        ChanFxKind::Bottom => candidate.price <= current.price,
    }
}

pub fn is_stronger_same_kind_fx(candidate: &ChanFx, current: &ChanFx) -> bool {
    match candidate.kind {
        ChanFxKind::Top => candidate.price > current.price,
        ChanFxKind::Bottom => candidate.price < current.price,
    }
}

fn fx_span_is_enough(start: &ChanFx, end: &ChanFx, min_merged_span: usize) -> bool {
    end.merged_index > start.merged_index
        && end.merged_index.saturating_sub(start.merged_index) >= min_merged_span
}

fn link_bis(bis: &mut [ChanBi]) {
    let len = bis.len();
    for (index, bi) in bis.iter_mut().enumerate() {
        bi.prev_index = index.checked_sub(1);
        bi.next_index = if index + 1 < len {
            Some(index + 1)
        } else {
            None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{build_bis, build_bis_with_min_span, normalize_fxs_for_bi};
    use crate::chan::model::{ChanDirection, ChanFx, ChanFxKind};

    #[test]
    fn fewer_than_two_fxs_have_no_bi() {
        let fxs = vec![fx(0, ChanFxKind::Bottom, 1, 1, 8.0)];
        assert!(build_bis(&fxs).is_empty());
    }

    #[test]
    fn adjacent_opposite_fxs_do_not_form_default_bi() {
        let fxs = vec![
            fx(0, ChanFxKind::Top, 1, 1, 12.0),
            fx(1, ChanFxKind::Bottom, 2, 2, 7.0),
        ];

        assert!(build_bis(&fxs).is_empty());
    }

    #[test]
    fn far_enough_opposite_fxs_form_single_up_bi() {
        let fxs = vec![
            fx(0, ChanFxKind::Bottom, 1, 10, 8.0),
            fx(1, ChanFxKind::Top, 5, 50, 12.0),
        ];

        let bis = build_bis(&fxs);

        assert_eq!(bis.len(), 1);
        assert_eq!(bis[0].index, 0);
        assert_eq!(bis[0].direction, ChanDirection::Up);
        assert_eq!(bis[0].start_fx_index, 0);
        assert_eq!(bis[0].end_fx_index, 1);
        assert_eq!(bis[0].start_bar_id, 10);
        assert_eq!(bis[0].start_price, 8.0);
        assert_eq!(bis[0].end_bar_id, 50);
        assert_eq!(bis[0].end_price, 12.0);
        assert!(bis[0].confirmed);
        assert_eq!(bis[0].prev_index, None);
        assert_eq!(bis[0].next_index, None);
    }

    #[test]
    fn far_enough_opposite_fxs_form_single_down_bi() {
        let fxs = vec![
            fx(0, ChanFxKind::Top, 1, 10, 12.0),
            fx(1, ChanFxKind::Bottom, 5, 50, 8.0),
        ];

        let bis = build_bis(&fxs);

        assert_eq!(bis.len(), 1);
        assert_eq!(bis[0].direction, ChanDirection::Down);
        assert_eq!(bis[0].start_fx_index, 0);
        assert_eq!(bis[0].end_fx_index, 1);
    }

    #[test]
    fn first_bi_uses_first_valid_free_opposite_fx() {
        let fxs = vec![
            fx(0, ChanFxKind::Top, 1, 10, 12.0),
            fx(1, ChanFxKind::Bottom, 2, 20, 8.5),
            fx(2, ChanFxKind::Top, 3, 30, 13.0),
            fx(3, ChanFxKind::Bottom, 6, 60, 6.5),
        ];

        let bis = build_bis(&fxs);

        assert_eq!(bis.len(), 1);
        assert_eq!(bis[0].start_fx_index, 0);
        assert_eq!(bis[0].start_bar_id, 10);
        assert_eq!(bis[0].start_price, 12.0);
        assert_eq!(bis[0].end_fx_index, 3);
        assert_eq!(bis[0].end_bar_id, 60);
        assert_eq!(bis[0].end_price, 6.5);
    }

    #[test]
    fn same_kind_normalizer_keeps_higher_top_but_bi_flow_keeps_free_candidates() {
        let fxs = vec![
            fx(0, ChanFxKind::Top, 1, 10, 10.0),
            fx(1, ChanFxKind::Top, 2, 20, 12.0),
            fx(2, ChanFxKind::Bottom, 6, 60, 8.0),
        ];

        let normalized = normalize_fxs_for_bi(&fxs, 4);
        let bis = build_bis(&fxs);

        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].index, 1);
        assert_eq!(bis.len(), 1);
        assert_eq!(bis[0].start_fx_index, 0);
        assert_eq!(bis[0].start_price, 10.0);
    }

    #[test]
    fn same_kind_normalizer_keeps_lower_bottom_but_bi_flow_keeps_free_candidates() {
        let fxs = vec![
            fx(0, ChanFxKind::Bottom, 1, 10, 8.0),
            fx(1, ChanFxKind::Bottom, 2, 20, 7.0),
            fx(2, ChanFxKind::Top, 6, 60, 12.0),
        ];

        let normalized = normalize_fxs_for_bi(&fxs, 4);
        let bis = build_bis(&fxs);

        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].index, 1);
        assert_eq!(bis.len(), 1);
        assert_eq!(bis[0].start_fx_index, 0);
        assert_eq!(bis[0].start_price, 8.0);
    }

    #[test]
    fn alternating_fxs_build_linked_bis() {
        let fxs = vec![
            fx(0, ChanFxKind::Bottom, 1, 10, 8.0),
            fx(1, ChanFxKind::Top, 5, 50, 12.0),
            fx(2, ChanFxKind::Bottom, 9, 90, 9.0),
            fx(3, ChanFxKind::Top, 13, 130, 13.0),
        ];

        let bis = build_bis(&fxs);

        assert_eq!(bis.len(), 3);
        assert_eq!(bis[0].direction, ChanDirection::Up);
        assert_eq!(bis[1].direction, ChanDirection::Down);
        assert_eq!(bis[2].direction, ChanDirection::Up);
        assert_eq!(bis[0].prev_index, None);
        assert_eq!(bis[0].next_index, Some(1));
        assert_eq!(bis[1].prev_index, Some(0));
        assert_eq!(bis[1].next_index, Some(2));
        assert_eq!(bis[2].prev_index, Some(1));
        assert_eq!(bis[2].next_index, None);
    }

    #[test]
    fn min_merged_span_can_filter_too_near_fxs() {
        let fxs = vec![
            fx(0, ChanFxKind::Bottom, 1, 10, 8.0),
            fx(1, ChanFxKind::Top, 2, 20, 12.0),
            fx(2, ChanFxKind::Bottom, 5, 50, 7.0),
        ];

        let bis = build_bis_with_min_span(&fxs, 4);

        assert_eq!(bis.len(), 0);
    }

    fn fx(index: usize, kind: ChanFxKind, merged_index: usize, bar_id: i64, price: f64) -> ChanFx {
        ChanFx {
            index,
            kind,
            merged_index,
            bar_id,
            price,
            confirmed: true,
            left_merged_index: merged_index.saturating_sub(1),
            center_merged_index: merged_index,
            right_merged_index: merged_index + 1,
        }
    }
}
