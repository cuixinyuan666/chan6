use crate::model::KLine1m;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeDirection {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedKLineBox {
    pub symbol: String,
    pub merged_index: usize,
    pub start_bar_id: i64,
    pub end_bar_id: i64,
    pub start_ts: i64,
    pub end_ts: i64,
    pub high: f64,
    pub low: f64,
    pub raw_count: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanFxKind {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanFxPoint {
    pub kind: ChanFxKind,
    pub merged_index: usize,
    pub raw_bar_id: i64,
    pub ts: i64,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanFxLine {
    pub from_kind: ChanFxKind,
    pub from_bar_id: i64,
    pub from_price: f64,
    pub to_kind: ChanFxKind,
    pub to_bar_id: i64,
    pub to_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanBasicOverlay {
    pub schema_version: u32,
    pub merged_boxes: Vec<MergedKLineBox>,
    pub fx_points: Vec<ChanFxPoint>,
    pub fx_lines: Vec<ChanFxLine>,
}

pub fn build_chan_basic_overlay(kline: &[KLine1m]) -> ChanBasicOverlay {
    let merged_boxes = merge_inclusion_boxes(kline);
    let fx_points = detect_fx_points(kline, &merged_boxes);
    let fx_lines = build_fx_lines(&fx_points);

    ChanBasicOverlay {
        schema_version: 1,
        merged_boxes,
        fx_points,
        fx_lines,
    }
}

fn merge_inclusion_boxes(kline: &[KLine1m]) -> Vec<MergedKLineBox> {
    let mut out: Vec<MergedKLineBox> = Vec::new();
    let mut direction = MergeDirection::Unknown;

    for bar in kline {
        let next = MergedKLineBox {
            symbol: bar.symbol.clone(),
            merged_index: out.len(),
            start_bar_id: bar.bar_id,
            end_bar_id: bar.bar_id,
            start_ts: bar.start_ts,
            end_ts: bar.end_ts,
            high: bar.high,
            low: bar.low,
            raw_count: 1,
        };

        if out.is_empty() {
            out.push(next);
            continue;
        }

        if is_inclusion(out.last().expect("non-empty"), &next) {
            let idx = out.len() - 1;
            if direction == MergeDirection::Unknown && out.len() >= 2 {
                direction = infer_direction(&out[out.len() - 2], &out[out.len() - 1]);
            }
            merge_into_last(&mut out[idx], &next, direction);
            continue;
        }

        let new_direction = infer_direction(out.last().expect("non-empty"), &next);
        if new_direction != MergeDirection::Unknown {
            direction = new_direction;
        }
        out.push(next);
    }

    for (idx, item) in out.iter_mut().enumerate() {
        item.merged_index = idx;
    }

    out
}

fn is_inclusion(a: &MergedKLineBox, b: &MergedKLineBox) -> bool {
    let b_inside_a = b.high <= a.high && b.low >= a.low;
    let a_inside_b = b.high >= a.high && b.low <= a.low;
    b_inside_a || a_inside_b
}

fn infer_direction(a: &MergedKLineBox, b: &MergedKLineBox) -> MergeDirection {
    if b.high > a.high && b.low > a.low {
        MergeDirection::Up
    } else if b.high < a.high && b.low < a.low {
        MergeDirection::Down
    } else {
        MergeDirection::Unknown
    }
}

fn merge_into_last(last: &mut MergedKLineBox, next: &MergedKLineBox, direction: MergeDirection) {
    match direction {
        MergeDirection::Up => {
            last.high = last.high.max(next.high);
            last.low = last.low.max(next.low);
        }
        MergeDirection::Down => {
            last.high = last.high.min(next.high);
            last.low = last.low.min(next.low);
        }
        MergeDirection::Unknown => {
            last.high = last.high.max(next.high);
            last.low = last.low.min(next.low);
        }
    }

    last.end_bar_id = next.end_bar_id;
    last.end_ts = next.end_ts;
    last.raw_count += next.raw_count;
}

fn detect_fx_points(kline: &[KLine1m], boxes: &[MergedKLineBox]) -> Vec<ChanFxPoint> {
    if boxes.len() < 3 {
        return Vec::new();
    }

    let mut out = Vec::new();

    for i in 1..(boxes.len() - 1) {
        let left = &boxes[i - 1];
        let mid = &boxes[i];
        let right = &boxes[i + 1];

        let is_top = mid.high > left.high && mid.high > right.high && mid.low > left.low && mid.low > right.low;
        let is_bottom = mid.low < left.low && mid.low < right.low && mid.high < left.high && mid.high < right.high;

        if is_top {
            out.push(build_fx_point(kline, mid, ChanFxKind::Top));
        } else if is_bottom {
            out.push(build_fx_point(kline, mid, ChanFxKind::Bottom));
        }
    }

    out
}

fn build_fx_point(kline: &[KLine1m], mid: &MergedKLineBox, kind: ChanFxKind) -> ChanFxPoint {
    let price = match kind {
        ChanFxKind::Top => mid.high,
        ChanFxKind::Bottom => mid.low,
    };

    let raw = kline
        .iter()
        .filter(|bar| bar.bar_id >= mid.start_bar_id && bar.bar_id <= mid.end_bar_id)
        .find(|bar| match kind {
            ChanFxKind::Top => nearly_equal(bar.high, price),
            ChanFxKind::Bottom => nearly_equal(bar.low, price),
        });

    let (raw_bar_id, ts) = raw
        .map(|bar| (bar.bar_id, bar.start_ts))
        .unwrap_or_else(|| ((mid.start_bar_id + mid.end_bar_id) / 2, mid.start_ts));

    ChanFxPoint {
        kind,
        merged_index: mid.merged_index,
        raw_bar_id,
        ts,
        price,
    }
}

fn nearly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

fn build_fx_lines(points: &[ChanFxPoint]) -> Vec<ChanFxLine> {
    points
        .windows(2)
        .filter_map(|pair| {
            let from = &pair[0];
            let to = &pair[1];
            if from.kind == to.kind {
                return None;
            }
            Some(ChanFxLine {
                from_kind: from.kind,
                from_bar_id: from.raw_bar_id,
                from_price: from.price,
                to_kind: to.kind,
                to_bar_id: to.raw_bar_id,
                to_price: to.price,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(id: i64, high: f64, low: f64) -> KLine1m {
        KLine1m {
            symbol: "000001".to_string(),
            bar_id: id,
            trading_day: 20260629,
            minute: 930 + id as i32,
            start_ts: id * 60,
            end_ts: id * 60 + 59,
            open: low,
            high,
            low,
            close: high,
            volume: 1.0,
            amount: 1.0,
            trade_count: 1,
        }
    }

    #[test]
    fn detects_basic_top_and_bottom_fx() {
        let rows = vec![
            bar(0, 10.0, 9.0),
            bar(1, 12.0, 11.0),
            bar(2, 11.0, 10.0),
            bar(3, 8.0, 7.0),
            bar(4, 9.0, 8.0),
        ];

        let overlay = build_chan_basic_overlay(&rows);
        assert_eq!(overlay.fx_points.len(), 2);
        assert_eq!(overlay.fx_points[0].kind, ChanFxKind::Top);
        assert_eq!(overlay.fx_points[0].raw_bar_id, 1);
        assert_eq!(overlay.fx_points[1].kind, ChanFxKind::Bottom);
        assert_eq!(overlay.fx_points[1].raw_bar_id, 3);
        assert_eq!(overlay.fx_lines.len(), 1);
    }

    #[test]
    fn merges_inclusion_boxes() {
        let rows = vec![
            bar(0, 10.0, 9.0),
            bar(1, 12.0, 11.0),
            bar(2, 11.5, 11.2),
            bar(3, 13.0, 12.0),
        ];

        let overlay = build_chan_basic_overlay(&rows);
        assert_eq!(overlay.merged_boxes.len(), 3);
        assert_eq!(overlay.merged_boxes[1].start_bar_id, 1);
        assert_eq!(overlay.merged_boxes[1].end_bar_id, 2);
        assert_eq!(overlay.merged_boxes[1].raw_count, 2);
    }
}
