use super::model::{ChanBi, ChanDirection, ChanSegment, CHAN_SEGMENT_N_LINE};

pub const DEFAULT_MIN_BI_COUNT_FOR_SEGMENT: usize = 3;

pub fn build_segments(bis: &[ChanBi]) -> Vec<ChanSegment> {
    build_segments_with_min_bi_count(bis, DEFAULT_MIN_BI_COUNT_FOR_SEGMENT)
}

pub fn build_segments_with_min_bi_count(bis: &[ChanBi], min_bi_count: usize) -> Vec<ChanSegment> {
    if bis.len() < min_bi_count || min_bi_count < DEFAULT_MIN_BI_COUNT_FOR_SEGMENT {
        return Vec::new();
    }

    let start = &bis[0];
    let end_index = latest_odd_bi_end_index(bis.len());
    let end = &bis[end_index];

    vec![ChanSegment {
        n: CHAN_SEGMENT_N_LINE,
        input_n: None,
        index: 0,
        direction: ChanDirection::from_prices(start.start_price, end.end_price),
        start_parent_index: Some(start.index),
        end_parent_index: Some(end.index),
        start_bar_id: start.start_bar_id,
        start_price: start.start_price,
        end_bar_id: end.end_bar_id,
        end_price: end.end_price,
        confirmed: false,
        reason: "chanpy_min_three_bi_link".to_string(),
    }]
}

fn latest_odd_bi_end_index(bi_count: usize) -> usize {
    debug_assert!(bi_count >= DEFAULT_MIN_BI_COUNT_FOR_SEGMENT);
    if bi_count % 2 == 1 {
        bi_count - 1
    } else {
        bi_count - 2
    }
}

#[cfg(test)]
mod tests {
    use super::build_segments;
    use crate::chan::model::{ChanBi, ChanDirection, CHAN_SEGMENT_N_LINE};

    #[test]
    fn fewer_than_three_bis_have_no_segment() {
        let bis = vec![bi(0, 1, 5, 8.0, 14.0), bi(1, 5, 9, 14.0, 6.5)];
        let segments = build_segments(&bis);
        assert!(segments.is_empty());
    }

    #[test]
    fn three_bis_form_minimal_unconfirmed_line_segment() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].n, CHAN_SEGMENT_N_LINE);
        assert_eq!(segments[0].index, 0);
        assert_eq!(segments[0].direction, ChanDirection::Up);
        assert_eq!(segments[0].start_parent_index, Some(0));
        assert_eq!(segments[0].end_parent_index, Some(2));
        assert_eq!(segments[0].start_bar_id, 1);
        assert_eq!(segments[0].start_price, 8.0);
        assert_eq!(segments[0].end_bar_id, 13);
        assert_eq!(segments[0].end_price, 15.0);
        assert!(!segments[0].confirmed);
        assert_eq!(segments[0].reason, "chanpy_min_three_bi_link");
    }

    #[test]
    fn five_bis_extend_the_unconfirmed_line_segment_to_latest_same_direction_endpoint() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
            bi(4, 17, 21, 5.8, 16.0),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].n, CHAN_SEGMENT_N_LINE);
        assert_eq!(segments[0].index, 0);
        assert_eq!(segments[0].direction, ChanDirection::Up);
        assert_eq!(segments[0].start_parent_index, Some(0));
        assert_eq!(segments[0].end_parent_index, Some(4));
        assert_eq!(segments[0].start_bar_id, 1);
        assert_eq!(segments[0].start_price, 8.0);
        assert_eq!(segments[0].end_bar_id, 21);
        assert_eq!(segments[0].end_price, 16.0);
        assert!(!segments[0].confirmed);
    }

    fn bi(
        index: usize,
        start_bar_id: i64,
        end_bar_id: i64,
        start_price: f64,
        end_price: f64,
    ) -> ChanBi {
        let mut item = ChanBi::new(
            index,
            index,
            index + 1,
            start_bar_id,
            start_price,
            end_bar_id,
            end_price,
        );
        item.prev_index = index.checked_sub(1);
        item.next_index = Some(index + 1);
        item
    }
}
