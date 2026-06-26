use super::model::{ChanBi, ChanFx, ChanFxKind};

/// Minimum distance between two FX merged indexes for first-stage BI construction.
///
/// This is intentionally aligned to chan.py strict BI behavior for the stage-1
/// fixture: adjacent top/bottom FX are not enough to form a BI.
///
/// First-stage rule is conservative and testable:
/// - BI is built from already detected FX, not raw bars;
/// - same-kind consecutive FX are normalized by keeping the stronger one;
/// - opposite-kind FX form a BI only when their merged indexes are far enough;
/// - output anchors always use `bar_id + price`.
pub const DEFAULT_MIN_BI_MERGED_SPAN: usize = 4;

pub fn build_bis(fxs: &[ChanFx]) -> Vec<ChanBi> {
    build_bis_with_min_span(fxs, DEFAULT_MIN_BI_MERGED_SPAN)
}

pub fn build_bis_with_min_span(fxs: &[ChanFx], min_merged_span: usize) -> Vec<ChanBi> {
    let normalized = normalize_fxs_for_bi(fxs, min_merged_span);
    let mut bis = Vec::new();

    for pair in normalized.windows(2) {
        let start = pair[0];
        let end = pair[1];
        if start.kind == end.kind {
            continue;
        }
        if !fx_span_is_enough(start, end, min_merged_span) {
            continue;
        }

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

    link_bis(&mut bis);
    bis
}

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
        bi.next_index = if index + 1 < len { Some(index + 1) } else { None };
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
    fn same_kind_top_keeps_higher_price() {
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
        assert_eq!(bis[0].start_fx_index, 1);
        assert_eq!(bis[0].start_price, 12.0);
    }

    #[test]
    fn same_kind_bottom_keeps_lower_price() {
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
        assert_eq!(bis[0].start_fx_index, 1);
        assert_eq!(bis[0].start_price, 7.0);
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

        let bis = build_bis_with_min_span(&fxs, 2);

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
