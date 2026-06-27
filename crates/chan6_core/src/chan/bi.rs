use super::model::{ChanBi, ChanFx, ChanFxKind};

/// Minimum distance between two FX merged indexes for first-stage BI construction.
///
/// This is intentionally aligned to chan.py strict BI behavior for the stage-1
/// fixtures: adjacent top/bottom FX are not enough to form a BI.
///
/// Stage-1 BI construction follows the observed hichan first-BI flow:
/// - before the first BI exists, FX are kept in a free candidate list;
/// - when a new FX arrives, the first opposite free FX that can make a BI wins;
/// - this means a later stronger same-kind FX must not blindly replace an earlier
///   free FX before can_make_bi has been tested;
/// - after the first BI, later BI candidates are built from the previous BI end;
/// - output anchors always use `bar_id + price`.
pub const DEFAULT_MIN_BI_MERGED_SPAN: usize = 4;

pub fn build_bis(fxs: &[ChanFx]) -> Vec<ChanBi> {
    build_bis_with_min_span(fxs, DEFAULT_MIN_BI_MERGED_SPAN)
}

pub fn build_bis_with_min_span(fxs: &[ChanFx], min_merged_span: usize) -> Vec<ChanBi> {
    let mut bis = Vec::new();
    let mut free_fxs: Vec<&ChanFx> = Vec::new();
    let mut last_bi_end: Option<&ChanFx> = None;

    for fx in fxs {
        if let Some(last_end) = last_bi_end {
            if fx.kind == last_end.kind {
                continue;
            }
            if fx_span_is_enough(last_end, fx, min_merged_span) {
                push_bi(&mut bis, last_end, fx);
                last_bi_end = Some(fx);
            }
            continue;
        }

        let mut first_valid_start = None;
        for existing in &free_fxs {
            if existing.kind == fx.kind {
                continue;
            }
            if fx_span_is_enough(existing, fx, min_merged_span) {
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

    link_bis(&mut bis);
    bis
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
/// Full BI construction should use `build_bis_with_min_span`, because chan.py's
/// first-BI free candidate flow is not equivalent to blindly normalizing all FX
/// before BI creation.
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
