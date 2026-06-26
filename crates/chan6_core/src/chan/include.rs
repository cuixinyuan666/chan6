use super::model::{ChanBar, ChanDirection, ChanMergedBar};

/// Convert raw Chan bars into inclusion-processed merged bars.
///
/// Business rule:
/// - calculation is Rust-authoritative;
/// - all output anchors remain `bar_id + price`;
/// - inclusion is processed before FX/BI detection;
/// - `high/low` are the raw visual envelope;
/// - `calc_high/calc_low` are hichan-style contracted values used by FX/BI.
pub fn merge_included_bars(bars: &[ChanBar]) -> Vec<ChanMergedBar> {
    let mut merged: Vec<ChanMergedBar> = Vec::new();

    for bar in bars {
        if merged.is_empty() {
            merged.push(ChanMergedBar::from_bar(0, bar));
            continue;
        }

        let last_index = merged.len() - 1;
        let next = ChanMergedBar::from_bar(last_index, bar);

        if !has_include_relation(&merged[last_index], &next) {
            merged.push(ChanMergedBar::from_bar(merged.len(), bar));
            continue;
        }

        let direction = infer_merge_direction(&merged, &next);
        let last = merged
            .pop()
            .expect("last merged bar exists when include relation is detected");
        let updated = merge_pair(last_index, &last, &next, direction);
        merged.push(updated);
    }

    renumber_merged_bars(&mut merged);
    merged
}

pub fn has_include_relation(left: &ChanMergedBar, right: &ChanMergedBar) -> bool {
    contains_price_range(left.calc_high, left.calc_low, right.calc_high, right.calc_low)
        || contains_price_range(right.calc_high, right.calc_low, left.calc_high, left.calc_low)
}

fn contains_price_range(outer_high: f64, outer_low: f64, inner_high: f64, inner_low: f64) -> bool {
    outer_high >= inner_high && outer_low <= inner_low
}

fn infer_merge_direction(merged: &[ChanMergedBar], next: &ChanMergedBar) -> ChanDirection {
    if merged.len() >= 2 {
        let prev = &merged[merged.len() - 2];
        let last = &merged[merged.len() - 1];
        let direction = direction_between_ranges(prev, last);
        if direction != ChanDirection::Unknown {
            return direction;
        }
    }

    let last = &merged[merged.len() - 1];
    let direction = direction_between_ranges(last, next);
    if direction != ChanDirection::Unknown {
        return direction;
    }

    ChanDirection::from_prices(last.close, next.close)
}

fn direction_between_ranges(left: &ChanMergedBar, right: &ChanMergedBar) -> ChanDirection {
    if right.calc_high > left.calc_high && right.calc_low > left.calc_low {
        ChanDirection::Up
    } else if right.calc_high < left.calc_high && right.calc_low < left.calc_low {
        ChanDirection::Down
    } else {
        ChanDirection::Unknown
    }
}

fn merge_pair(
    index: usize,
    left: &ChanMergedBar,
    right: &ChanMergedBar,
    direction: ChanDirection,
) -> ChanMergedBar {
    let effective_direction = match direction {
        ChanDirection::Unknown => ChanDirection::from_prices(left.close, right.close),
        direction => direction,
    };

    let (high, high_bar_id) = choose_raw_higher_high(left, right);
    let (low, low_bar_id) = choose_raw_lower_low(left, right);

    let (calc_high, calc_high_bar_id) = match effective_direction {
        ChanDirection::Down => choose_calc_lower_high(left, right),
        ChanDirection::Up | ChanDirection::Unknown => choose_calc_higher_high(left, right),
    };
    let (calc_low, calc_low_bar_id) = match effective_direction {
        ChanDirection::Down => choose_calc_lower_low(left, right),
        ChanDirection::Up | ChanDirection::Unknown => choose_calc_higher_low(left, right),
    };

    ChanMergedBar {
        index,
        symbol: left.symbol.clone(),
        start_bar_id: left.start_bar_id,
        end_bar_id: right.end_bar_id,
        high_bar_id,
        low_bar_id,
        calc_high_bar_id,
        calc_low_bar_id,
        trading_day: right.trading_day,
        minute: right.minute,
        start_ts: left.start_ts,
        end_ts: right.end_ts,
        open: left.open,
        high,
        low,
        calc_high,
        calc_low,
        close: right.close,
        volume: left.volume + right.volume,
        amount: left.amount + right.amount,
        trade_count: left.trade_count + right.trade_count,
    }
}

fn choose_raw_higher_high(left: &ChanMergedBar, right: &ChanMergedBar) -> (f64, i64) {
    if right.high > left.high {
        (right.high, right.high_bar_id)
    } else {
        (left.high, left.high_bar_id)
    }
}

fn choose_raw_lower_low(left: &ChanMergedBar, right: &ChanMergedBar) -> (f64, i64) {
    if right.low < left.low {
        (right.low, right.low_bar_id)
    } else {
        (left.low, left.low_bar_id)
    }
}

fn choose_calc_higher_high(left: &ChanMergedBar, right: &ChanMergedBar) -> (f64, i64) {
    if right.calc_high >= left.calc_high {
        (right.calc_high, right.calc_high_bar_id)
    } else {
        (left.calc_high, left.calc_high_bar_id)
    }
}

fn choose_calc_lower_high(left: &ChanMergedBar, right: &ChanMergedBar) -> (f64, i64) {
    if right.calc_high <= left.calc_high {
        (right.calc_high, right.calc_high_bar_id)
    } else {
        (left.calc_high, left.calc_high_bar_id)
    }
}

fn choose_calc_higher_low(left: &ChanMergedBar, right: &ChanMergedBar) -> (f64, i64) {
    if right.calc_low >= left.calc_low {
        (right.calc_low, right.calc_low_bar_id)
    } else {
        (left.calc_low, left.calc_low_bar_id)
    }
}

fn choose_calc_lower_low(left: &ChanMergedBar, right: &ChanMergedBar) -> (f64, i64) {
    if right.calc_low <= left.calc_low {
        (right.calc_low, right.calc_low_bar_id)
    } else {
        (left.calc_low, left.calc_low_bar_id)
    }
}

fn renumber_merged_bars(merged: &mut [ChanMergedBar]) {
    for (index, bar) in merged.iter_mut().enumerate() {
        bar.index = index;
    }
}

#[cfg(test)]
mod tests {
    use super::{has_include_relation, merge_included_bars};
    use crate::chan::model::{ChanBar, ChanMergedBar};

    #[test]
    fn non_included_bars_are_preserved() {
        let bars = vec![bar(1, 10.0, 8.0), bar(2, 11.0, 9.0), bar(3, 12.0, 10.0)];

        let merged = merge_included_bars(&bars);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].start_bar_id, 1);
        assert_eq!(merged[2].end_bar_id, 3);
    }

    #[test]
    fn included_bar_is_merged_in_up_direction() {
        let bars = vec![
            bar(1, 10.0, 8.0),
            bar(2, 11.0, 9.0),
            bar(3, 10.5, 9.5),
        ];

        let merged = merge_included_bars(&bars);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[1].start_bar_id, 2);
        assert_eq!(merged[1].end_bar_id, 3);
        assert_eq!(merged[1].high, 11.0);
        assert_eq!(merged[1].low, 9.0);
        assert_eq!(merged[1].calc_high, 11.0);
        assert_eq!(merged[1].calc_low, 9.5);
        assert_eq!(merged[1].high_bar_id, 2);
        assert_eq!(merged[1].low_bar_id, 2);
        assert_eq!(merged[1].calc_high_bar_id, 2);
        assert_eq!(merged[1].calc_low_bar_id, 3);
    }

    #[test]
    fn included_bar_is_merged_in_down_direction() {
        let bars = vec![
            bar(1, 12.0, 10.0),
            bar(2, 11.0, 9.0),
            bar(3, 10.5, 9.5),
        ];

        let merged = merge_included_bars(&bars);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[1].start_bar_id, 2);
        assert_eq!(merged[1].end_bar_id, 3);
        assert_eq!(merged[1].high, 11.0);
        assert_eq!(merged[1].low, 9.0);
        assert_eq!(merged[1].calc_high, 10.5);
        assert_eq!(merged[1].calc_low, 9.0);
        assert_eq!(merged[1].high_bar_id, 2);
        assert_eq!(merged[1].low_bar_id, 2);
        assert_eq!(merged[1].calc_high_bar_id, 3);
        assert_eq!(merged[1].calc_low_bar_id, 2);
    }

    #[test]
    fn expanding_bar_can_contain_previous_merged_bar() {
        let bars = vec![
            bar(1, 10.0, 8.0),
            bar(2, 11.0, 9.0),
            bar(3, 12.0, 8.5),
        ];

        let merged = merge_included_bars(&bars);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[1].start_bar_id, 2);
        assert_eq!(merged[1].end_bar_id, 3);
        assert_eq!(merged[1].high, 12.0);
        assert_eq!(merged[1].low, 8.5);
        assert_eq!(merged[1].calc_high, 12.0);
        assert_eq!(merged[1].calc_low, 9.0);
        assert_eq!(merged[1].high_bar_id, 3);
        assert_eq!(merged[1].low_bar_id, 3);
        assert_eq!(merged[1].calc_high_bar_id, 3);
        assert_eq!(merged[1].calc_low_bar_id, 2);
    }

    #[test]
    fn include_relation_detects_both_directions() {
        let a = ChanMergedBar::from_bar(0, &bar(1, 10.0, 8.0));
        let b = ChanMergedBar::from_bar(1, &bar(2, 9.5, 8.5));
        let c = ChanMergedBar::from_bar(2, &bar(3, 11.0, 7.5));

        assert!(has_include_relation(&a, &b));
        assert!(has_include_relation(&b, &a));
        assert!(has_include_relation(&a, &c));
        assert!(has_include_relation(&c, &a));
    }

    fn bar(bar_id: i64, high: f64, low: f64) -> ChanBar {
        ChanBar {
            symbol: "002003".to_string(),
            bar_id,
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
