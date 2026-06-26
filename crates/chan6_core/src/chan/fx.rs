use super::model::{ChanFx, ChanFxKind, ChanMergedBar};

/// Detect top/bottom fractals from inclusion-processed merged bars.
///
/// Strict rule after inclusion handling:
/// - top FX: center.high > left.high && center.high > right.high
///           && center.low > left.low && center.low > right.low
/// - bottom FX: center.low < left.low && center.low < right.low
///              && center.high < left.high && center.high < right.high
///
/// Output anchors always use `bar_id + price`.
pub fn detect_fxs(merged_bars: &[ChanMergedBar]) -> Vec<ChanFx> {
    if merged_bars.len() < 3 {
        return Vec::new();
    }

    let mut fxs = Vec::new();
    for center_index in 1..(merged_bars.len() - 1) {
        let left = &merged_bars[center_index - 1];
        let center = &merged_bars[center_index];
        let right = &merged_bars[center_index + 1];

        let Some(kind) = classify_fx(left, center, right) else {
            continue;
        };

        let (bar_id, price) = match kind {
            ChanFxKind::Top => (center.high_bar_id, center.high),
            ChanFxKind::Bottom => (center.low_bar_id, center.low),
        };

        fxs.push(ChanFx {
            index: fxs.len(),
            kind,
            merged_index: center.index,
            bar_id,
            price,
            confirmed: true,
            left_merged_index: left.index,
            center_merged_index: center.index,
            right_merged_index: right.index,
        });
    }

    fxs
}

pub fn classify_fx(
    left: &ChanMergedBar,
    center: &ChanMergedBar,
    right: &ChanMergedBar,
) -> Option<ChanFxKind> {
    if is_top_fx(left, center, right) {
        Some(ChanFxKind::Top)
    } else if is_bottom_fx(left, center, right) {
        Some(ChanFxKind::Bottom)
    } else {
        None
    }
}

pub fn is_top_fx(left: &ChanMergedBar, center: &ChanMergedBar, right: &ChanMergedBar) -> bool {
    center.high > left.high
        && center.high > right.high
        && center.low > left.low
        && center.low > right.low
}

pub fn is_bottom_fx(left: &ChanMergedBar, center: &ChanMergedBar, right: &ChanMergedBar) -> bool {
    center.low < left.low
        && center.low < right.low
        && center.high < left.high
        && center.high < right.high
}

#[cfg(test)]
mod tests {
    use super::{classify_fx, detect_fxs, is_bottom_fx, is_top_fx};
    use crate::chan::model::{ChanFxKind, ChanMergedBar};

    #[test]
    fn fewer_than_three_merged_bars_have_no_fx() {
        let merged = vec![merged_bar(0, 1, 10.0, 8.0), merged_bar(1, 2, 11.0, 9.0)];

        assert!(detect_fxs(&merged).is_empty());
    }

    #[test]
    fn detects_top_fx_from_three_merged_bars() {
        let merged = vec![
            merged_bar(0, 1, 10.0, 8.0),
            merged_bar(1, 2, 12.0, 10.0),
            merged_bar(2, 3, 11.0, 9.0),
        ];

        let fxs = detect_fxs(&merged);

        assert_eq!(fxs.len(), 1);
        assert_eq!(fxs[0].kind, ChanFxKind::Top);
        assert_eq!(fxs[0].merged_index, 1);
        assert_eq!(fxs[0].bar_id, 2);
        assert_eq!(fxs[0].price, 12.0);
        assert!(fxs[0].confirmed);
        assert!(fxs[0].is_top());
    }

    #[test]
    fn detects_bottom_fx_from_three_merged_bars() {
        let merged = vec![
            merged_bar(0, 1, 12.0, 10.0),
            merged_bar(1, 2, 11.0, 8.0),
            merged_bar(2, 3, 13.0, 9.0),
        ];

        let fxs = detect_fxs(&merged);

        assert_eq!(fxs.len(), 1);
        assert_eq!(fxs[0].kind, ChanFxKind::Bottom);
        assert_eq!(fxs[0].merged_index, 1);
        assert_eq!(fxs[0].bar_id, 2);
        assert_eq!(fxs[0].price, 8.0);
        assert!(fxs[0].confirmed);
        assert!(fxs[0].is_bottom());
    }

    #[test]
    fn equal_high_or_low_does_not_form_strict_fx() {
        let equal_high = vec![
            merged_bar(0, 1, 10.0, 8.0),
            merged_bar(1, 2, 10.0, 9.0),
            merged_bar(2, 3, 9.0, 7.0),
        ];
        let equal_low = vec![
            merged_bar(0, 1, 11.0, 8.0),
            merged_bar(1, 2, 10.0, 8.0),
            merged_bar(2, 3, 12.0, 9.0),
        ];

        assert!(detect_fxs(&equal_high).is_empty());
        assert!(detect_fxs(&equal_low).is_empty());
    }

    #[test]
    fn detects_multiple_alternating_fxs() {
        let merged = vec![
            merged_bar(0, 1, 10.0, 8.0),
            merged_bar(1, 2, 12.0, 10.0),
            merged_bar(2, 3, 11.0, 7.0),
            merged_bar(3, 4, 13.0, 9.0),
            merged_bar(4, 5, 12.0, 8.0),
        ];

        let fxs = detect_fxs(&merged);

        assert_eq!(fxs.len(), 3);
        assert_eq!(fxs[0].kind, ChanFxKind::Top);
        assert_eq!(fxs[1].kind, ChanFxKind::Bottom);
        assert_eq!(fxs[2].kind, ChanFxKind::Top);
        assert_eq!(fxs[0].index, 0);
        assert_eq!(fxs[1].index, 1);
        assert_eq!(fxs[2].index, 2);
    }

    #[test]
    fn classifier_matches_top_bottom_helpers() {
        let left = merged_bar(0, 1, 10.0, 8.0);
        let top = merged_bar(1, 2, 12.0, 10.0);
        let right = merged_bar(2, 3, 11.0, 9.0);
        assert!(is_top_fx(&left, &top, &right));
        assert_eq!(classify_fx(&left, &top, &right), Some(ChanFxKind::Top));

        let left = merged_bar(0, 1, 12.0, 10.0);
        let bottom = merged_bar(1, 2, 11.0, 8.0);
        let right = merged_bar(2, 3, 13.0, 9.0);
        assert!(is_bottom_fx(&left, &bottom, &right));
        assert_eq!(
            classify_fx(&left, &bottom, &right),
            Some(ChanFxKind::Bottom)
        );
    }

    fn merged_bar(index: usize, bar_id: i64, high: f64, low: f64) -> ChanMergedBar {
        ChanMergedBar {
            index,
            symbol: "002003".to_string(),
            start_bar_id: bar_id,
            end_bar_id: bar_id,
            high_bar_id: bar_id,
            low_bar_id: bar_id,
            trading_day: 20260511,
            minute: 930 + bar_id as i32,
            start_ts: bar_id * 60,
            end_ts: bar_id * 60 + 59,
            open: low,
            high,
            low,
            close: high,
            volume: 100.0,
            amount: 1000.0,
            trade_count: 10,
        }
    }
}
