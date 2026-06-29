use super::model::{ChanBi, ChanSegment};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChanZs {
    pub index: usize,
    pub start_bi_index: usize,
    pub end_bi_index: usize,
    pub start_bar_id: i64,
    pub end_bar_id: i64,
    pub zg: f64,
    pub zd: f64,
    pub gg: f64,
    pub dd: f64,
    pub confirmed: bool,
    pub parent_segment_index: Option<usize>,
}

pub fn build_zs(bis: &[ChanBi], segments: &[ChanSegment]) -> Vec<ChanZs> {
    let mut result = Vec::new();

    for segment in segments {
        let Some(segment_start_bi_index) = segment.start_parent_index else {
            continue;
        };
        let Some(segment_end_bi_index) = segment.end_parent_index else {
            continue;
        };

        if segment_end_bi_index >= bis.len() || segment_start_bi_index >= bis.len() {
            continue;
        }

        // hichan/chan.py normal ZS is calculated inside a segment body.
        // The segment entry BI and exit BI are not part of the first center.
        let inner_start = segment_start_bi_index + 1;
        let inner_end = segment_end_bi_index.saturating_sub(1);

        if inner_start + 2 > inner_end {
            continue;
        }

        let mut cursor = inner_start;
        while cursor + 2 <= inner_end {
            let Some((zg, zd, mut gg, mut dd)) = initial_three_bi_zs_range(bis, cursor)
            else {
                cursor += 1;
                continue;
            };

            let mut end_bi_index = cursor + 2;
            let mut next_bi_index = end_bi_index + 1;

            while next_bi_index <= inner_end {
                let (low, high) = bi_range(&bis[next_bi_index]);

                // Extend while the next BI still overlaps the fixed initial
                // center interval [zd, zg]. The center interval itself remains
                // anchored by the first three BIs; gg/dd expand as envelope.
                if high < zd || low > zg {
                    break;
                }

                gg = gg.max(high);
                dd = dd.min(low);
                end_bi_index = next_bi_index;
                next_bi_index += 1;
            }

            result.push(ChanZs {
                index: result.len(),
                start_bi_index: cursor,
                end_bi_index,
                start_bar_id: bis[cursor].start_bar_id,
                end_bar_id: bis[end_bi_index].end_bar_id,
                zg,
                zd,
                gg,
                dd,
                confirmed: false,
                parent_segment_index: Some(segment.index),
            });

            cursor = end_bi_index + 1;
        }
    }

    result
}

fn initial_three_bi_zs_range(bis: &[ChanBi], start: usize) -> Option<(f64, f64, f64, f64)> {
    let mut zg = f64::INFINITY;
    let mut zd = f64::NEG_INFINITY;
    let mut gg = f64::NEG_INFINITY;
    let mut dd = f64::INFINITY;

    for bi in &bis[start..=start + 2] {
        let (low, high) = bi_range(bi);
        zg = zg.min(high);
        zd = zd.max(low);
        gg = gg.max(high);
        dd = dd.min(low);
    }

    if zd <= zg {
        Some((zg, zd, gg, dd))
    } else {
        None
    }
}

fn bi_range(bi: &ChanBi) -> (f64, f64) {
    (
        bi.start_price.min(bi.end_price),
        bi.start_price.max(bi.end_price),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chan::model::{ChanBi, ChanDirection, ChanSegment, CHAN_SEGMENT_N_LINE};

    #[test]
    fn builds_zs_from_segment_inner_three_bi_overlap() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
            bi(4, 17, 21, 5.8, 14.2),
            bi(5, 21, 25, 14.2, 4.0),
            bi(6, 25, 29, 4.0, 13.5),
            bi(7, 29, 33, 13.5, 3.0),
            bi(8, 33, 37, 3.0, 12.5),
            bi(9, 37, 41, 12.5, 2.5),
        ];

        let segments = vec![segment(0, 3, 9)];
        let zs = build_zs(&bis, &segments);

        assert_eq!(zs.len(), 1);
        assert_eq!(zs[0].start_bi_index, 4);
        assert_eq!(zs[0].end_bi_index, 8);
        assert_eq!(zs[0].start_bar_id, 17);
        assert_eq!(zs[0].end_bar_id, 37);
        assert_close(zs[0].zg, 13.5);
        assert_close(zs[0].zd, 5.8);
        assert_close(zs[0].gg, 14.2);
        assert_close(zs[0].dd, 3.0);
        assert_eq!(zs[0].parent_segment_index, Some(0));
    }

    fn bi(index: usize, start_bar_id: i64, end_bar_id: i64, start_price: f64, end_price: f64) -> ChanBi {
        ChanBi {
            index,
            direction: if end_price >= start_price {
                ChanDirection::Up
            } else {
                ChanDirection::Down
            },
            start_fx_index: index,
            end_fx_index: index + 1,
            start_bar_id,
            start_price,
            end_bar_id,
            end_price,
            confirmed: true,
            prev_index: None,
            next_index: None,
            parent_segment_index: None,
        }
    }

    fn segment(index: usize, start_bi_index: usize, end_bi_index: usize) -> ChanSegment {
        ChanSegment {
            index,
            n: CHAN_SEGMENT_N_LINE,
            input_n: None,
            direction: ChanDirection::Down,
            start_parent_index: Some(start_bi_index),
            end_parent_index: Some(end_bi_index),
            start_bar_id: 0,
            start_price: 0.0,
            end_bar_id: 0,
            end_price: 0.0,
            confirmed: false,
            reason: "unit_test".to_string(),
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "actual={actual}, expected={expected}"
        );
    }
}
